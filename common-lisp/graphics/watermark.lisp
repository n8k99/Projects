;;;; Watermarking Application — LSB steganography + visible overlay + position ADT.

(defpackage #:watermark
  (:use #:cl)
  (:export #:lsb-capacity #:embed-lsb #:extract-lsb
           #:apply-visible-watermark #:resolve-position
           #:+top-left+ #:+top-right+ #:+bottom-left+
           #:+bottom-right+ #:+center+ #:+tiled+))

(in-package #:watermark)

(defconstant +top-left+ :top-left)
(defconstant +top-right+ :top-right)
(defconstant +bottom-left+ :bottom-left)
(defconstant +bottom-right+ :bottom-right)
(defconstant +center+ :center)
(defconstant +tiled+ :tiled)

(define-condition watermark-error (error)
  ((reason :initarg :reason :reader watermark-error-reason)))

(defun signal-watermark-error (reason)
  (error 'watermark-error :reason reason))

(defun lsb-capacity (image bits-per-channel)
  (cond
    ((or (<= bits-per-channel 0) (> bits-per-channel 4)) 0)
    (t
     (let ((total-bits (* (length (grayscale::image-pixels image)) 3 bits-per-channel)))
       (max 0 (- (floor total-bits 8) 4))))))

(defun byte-to-bits (b)
  (loop for i from 7 downto 0
        collect (logand (ash b (- i)) 1)))

(defun embed-lsb (image payload bits-per-channel)
  (when (or (<= bits-per-channel 0) (> bits-per-channel 4))
    (signal-watermark-error :invalid-bits-per-channel))
  (let ((cap (lsb-capacity image bits-per-channel)))
    (when (> (length payload) cap)
      (signal-watermark-error :payload-too-large)))

  (let* ((payload-len (length payload))
         (len-bytes (list (logand (ash payload-len -24) 255)
                           (logand (ash payload-len -16) 255)
                           (logand (ash payload-len -8) 255)
                           (logand payload-len 255)))
         (bits (mapcan #'byte-to-bits (append len-bytes (coerce payload 'list))))
         (mask (logand (lognot (1- (ash 1 bits-per-channel))) 255))
         (bit-idx 0)
         (total-bits (length bits))
         (pixels-arr (coerce (grayscale::image-pixels image) 'vector))
         (new-pixels nil))
    (flet ((consume-chunk ()
              (let ((chunk 0))
                (loop for i from 0 below bits-per-channel
                      while (< bit-idx total-bits) do
                      (setf chunk (logior (ash chunk 1) (nth bit-idx bits)))
                      (incf bit-idx))
                chunk)))
      (loop for i from 0 below (length pixels-arr) do
            (let ((pixel (aref pixels-arr i)))
              (if (>= bit-idx total-bits)
                  (push pixel new-pixels)
                  (let* ((r (if (< bit-idx total-bits)
                                (logior (logand (grayscale::rgb-r pixel) mask) (consume-chunk))
                                (grayscale::rgb-r pixel)))
                         (g (if (< bit-idx total-bits)
                                (logior (logand (grayscale::rgb-g pixel) mask) (consume-chunk))
                                (grayscale::rgb-g pixel)))
                         (b (if (< bit-idx total-bits)
                                (logior (logand (grayscale::rgb-b pixel) mask) (consume-chunk))
                                (grayscale::rgb-b pixel))))
                    (push (grayscale::make-rgb :r r :g g :b b) new-pixels))))))
    (grayscale::make-image :width (grayscale::image-width image)
                            :height (grayscale::image-height image)
                            :pixels (nreverse new-pixels))))

(defun extract-lsb (image bits-per-channel)
  (when (or (<= bits-per-channel 0) (> bits-per-channel 4))
    (signal-watermark-error :invalid-bits-per-channel))
  (let* ((mask (1- (ash 1 bits-per-channel)))
         (bits nil))
    (dolist (pixel (grayscale::image-pixels image))
      (dolist (ch (list (grayscale::rgb-r pixel)
                        (grayscale::rgb-g pixel)
                        (grayscale::rgb-b pixel)))
        (let ((chunk (logand ch mask)))
          (loop for i from 0 below bits-per-channel do
                (push (logand (ash chunk (- (- bits-per-channel 1 i))) 1) bits)))))
    (setf bits (nreverse bits))
    (when (< (length bits) 32)
      (signal-watermark-error :truncated-data))
    (let* ((len-bytes (make-array 4 :initial-element 0))
           (payload-len 0))
      (loop for i from 0 below 32 do
            (let ((byte-idx (floor i 8)))
              (setf (aref len-bytes byte-idx)
                    (logior (ash (aref len-bytes byte-idx) 1) (nth i bits)))))
      (loop for i from 0 below 4 do
            (setf payload-len (logior (ash payload-len 8) (aref len-bytes i))))
      (let ((total-bits-needed (+ 32 (* payload-len 8))))
        (when (< (length bits) total-bits-needed)
          (signal-watermark-error :truncated-data)))
      (let ((payload (make-array payload-len :element-type '(unsigned-byte 8))))
        (loop for byte-i from 0 below payload-len do
              (let ((b 0))
                (loop for bit-j from 0 below 8 do
                      (setf b (logior (ash b 1)
                                       (nth (+ 32 (* byte-i 8) bit-j) bits))))
                (setf (aref payload byte-i) b)))
        payload))))

(defun resolve-position (base wm pos)
  (let ((bw (grayscale::image-width base))
        (bh (grayscale::image-height base))
        (ww (grayscale::image-width wm))
        (wh (grayscale::image-height wm)))
    (case pos
      (:top-left (cons 0 0))
      (:top-right (cons (max 0 (- bw ww)) 0))
      (:bottom-left (cons 0 (max 0 (- bh wh))))
      (:bottom-right (cons (max 0 (- bw ww)) (max 0 (- bh wh))))
      (:center (cons (floor (- bw ww) 2) (floor (- bh wh) 2)))
      (:tiled (cons 0 0)))))

(defun blend-channel (a b alpha)
  (floor (+ (* a (- 255 alpha)) (* b alpha)) 255))

(defun blend-region (out x0 y0 wm opacity)
  (loop for wy from 0 below (grayscale::image-height wm) do
        (loop for wx from 0 below (grayscale::image-width wm) do
              (let ((bx (+ x0 wx))
                    (by (+ y0 wy)))
                (when (and (>= bx 0) (>= by 0)
                            (< bx (grayscale::image-width out))
                            (< by (grayscale::image-height out)))
                  (let* ((base-idx (+ (* by (grayscale::image-width out)) bx))
                         (wm-idx (+ (* wy (grayscale::image-width wm)) wx))
                         (bp (nth base-idx (grayscale::image-pixels out)))
                         (wp (nth wm-idx (grayscale::image-pixels wm)))
                         (new-p (grayscale::make-rgb
                                  :r (blend-channel (grayscale::rgb-r bp)
                                                     (grayscale::rgb-r wp) opacity)
                                  :g (blend-channel (grayscale::rgb-g bp)
                                                     (grayscale::rgb-g wp) opacity)
                                  :b (blend-channel (grayscale::rgb-b bp)
                                                     (grayscale::rgb-b wp) opacity))))
                    (setf (nth base-idx (grayscale::image-pixels out)) new-p)))))))

(defun apply-visible-watermark (base watermark position opacity)
  (let ((out (grayscale::make-image
               :width (grayscale::image-width base)
               :height (grayscale::image-height base)
               :pixels (copy-list (grayscale::image-pixels base)))))
    (case position
      (:tiled
       (loop for y from 0 below (grayscale::image-height base) by (grayscale::image-height watermark) do
             (loop for x from 0 below (grayscale::image-width base) by (grayscale::image-width watermark) do
                   (blend-region out x y watermark opacity))))
      (t
       (let ((xy (resolve-position base watermark position)))
         (blend-region out (car xy) (cdr xy) watermark opacity))))
    out))

(defun demo ()
  (let* ((base (grayscale::make-image
                :width 32 :height 32
                :pixels (loop repeat (* 32 32)
                                collect (grayscale::make-rgb :r 128 :g 128 :b 128)))))
    (format t "capacity @ 1 bit/chan: ~D bytes~%" (lsb-capacity base 1))
    (format t "capacity @ 2 bits/chan: ~D bytes~%" (lsb-capacity base 2))
    (format t "capacity @ 4 bits/chan: ~D bytes~%" (lsb-capacity base 4))

    (let* ((payload (map '(vector (unsigned-byte 8)) #'char-code "Hello, watermark!"))
           (embedded (embed-lsb base payload 1))
           (extracted (extract-lsb embedded 1))
           (ok (equalp extracted payload)))
      (format t "roundtrip ok: ~A~%" (if ok "True" "False"))
      (format t "extracted: ~A~%" (map 'string #'code-char extracted)))

    (let* ((wm (grayscale::make-image
                :width 8 :height 8
                :pixels (loop repeat 64
                                collect (grayscale::make-rgb :r 255 :g 0 :b 0))))
           (tl (apply-visible-watermark base wm :top-left 255))
           (br (apply-visible-watermark base wm :bottom-right 255))
           (tr (apply-visible-watermark base wm :top-right 128))
           (tiled (apply-visible-watermark base wm :tiled 255)))
      (let ((p (nth 0 (grayscale::image-pixels tl))))
        (format t "top-left (0,0): Rgb(r=~D, g=~D, b=~D)~%"
                (grayscale::rgb-r p) (grayscale::rgb-g p) (grayscale::rgb-b p)))
      (let ((p (nth (+ (* 31 32) 31) (grayscale::image-pixels br))))
        (format t "bottom-right (31,31): Rgb(r=~D, g=~D, b=~D)~%"
                (grayscale::rgb-r p) (grayscale::rgb-g p) (grayscale::rgb-b p)))
      (let ((p (nth 31 (grayscale::image-pixels tr))))
        (format t "top-right (31,0) blended: Rgb(r=~D, g=~D, b=~D)~%"
                (grayscale::rgb-r p) (grayscale::rgb-g p) (grayscale::rgb-b p)))
      (let ((all-red t))
        (dolist (p (grayscale::image-pixels tiled))
          (unless (and (= (grayscale::rgb-r p) 255)
                        (= (grayscale::rgb-g p) 0)
                        (= (grayscale::rgb-b p) 0))
            (setf all-red nil)))
        (format t "tiled all red: ~A~%" (if all-red "True" "False"))))))
