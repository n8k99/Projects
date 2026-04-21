;;;; MP3-to-Wav Converter — audio buffer + resample + normalize + RIFF.

(defpackage #:mp3-wav-converter
  (:use #:cl)
  (:export #:make-audio-buffer #:audio-buffer-sample-rate
           #:audio-buffer-channels #:audio-buffer-samples
           #:audio-buffer-frame-count #:audio-buffer-duration-ms
           #:make-mp3-frame #:decode-mp3-to-buffer
           #:resample #:normalize-to-peak #:to-mono #:to-stereo
           #:encode-wav #:decode-wav))

(in-package #:mp3-wav-converter)

(defstruct audio-buffer
  (sample-rate 0 :type integer)
  (channels 0 :type integer)
  (samples nil :type list))

(defun audio-buffer-frame-count (buf)
  (if (zerop (audio-buffer-channels buf)) 0
      (floor (length (audio-buffer-samples buf))
              (audio-buffer-channels buf))))

(defun audio-buffer-duration-ms (buf)
  (if (zerop (audio-buffer-sample-rate buf)) 0
      (floor (* (audio-buffer-frame-count buf) 1000)
              (audio-buffer-sample-rate buf))))

(defstruct mp3-frame
  (sample-rate 0 :type integer)
  (channels 0 :type integer)
  (bitrate-kbps 0 :type integer)
  (samples-per-frame 0 :type integer)
  (pcm nil :type list))

(define-condition codec-error (error)
  ((reason :initarg :reason :reader codec-error-reason)))

(defun signal-codec-error (reason)
  (error 'codec-error :reason reason))

(defun decode-mp3-to-buffer (frames)
  (unless frames (signal-codec-error :no-frames))
  (let ((sample-rate (mp3-frame-sample-rate (first frames)))
        (channels (mp3-frame-channels (first frames))))
    (when (zerop sample-rate) (signal-codec-error :sample-rate-zero))
    (unless (or (= channels 1) (= channels 2))
      (signal-codec-error :unsupported-channels))
    (dolist (f frames)
      (when (or (/= (mp3-frame-sample-rate f) sample-rate)
                (/= (mp3-frame-channels f) channels))
        (signal-codec-error :mixed-formats)))
    (let ((samples (apply #'append (mapcar #'mp3-frame-pcm frames))))
      (make-audio-buffer :sample-rate sample-rate :channels channels
                          :samples samples))))

(defun clamp-i16 (v)
  (cond ((> v 32767) 32767)
        ((< v -32768) -32768)
        (t v)))

(defun resample (buf dst-rate)
  (when (or (zerop (audio-buffer-sample-rate buf)) (zerop dst-rate))
    (signal-codec-error :sample-rate-zero))
  (when (= dst-rate (audio-buffer-sample-rate buf))
    (return-from resample (make-audio-buffer
                            :sample-rate (audio-buffer-sample-rate buf)
                            :channels (audio-buffer-channels buf)
                            :samples (copy-list (audio-buffer-samples buf)))))
  (let ((in-frames (audio-buffer-frame-count buf))
        (channels (audio-buffer-channels buf))
        (src-rate (audio-buffer-sample-rate buf))
        (samples-arr (coerce (audio-buffer-samples buf) 'vector)))
    (when (zerop in-frames)
      (return-from resample (make-audio-buffer
                              :sample-rate dst-rate
                              :channels channels :samples nil)))
    (let* ((out-frames (floor (+ (* in-frames dst-rate) (floor src-rate 2)) src-rate))
           (out nil))
      (loop for i from 0 below out-frames do
            (let* ((src-pos (floor (* i src-rate 1000) dst-rate))
                   (src-frame (floor src-pos 1000))
                   (frac (mod src-pos 1000))
                   (next-frame (min (1+ src-frame) (1- in-frames))))
              (loop for c from 0 below channels do
                    (let* ((a (aref samples-arr (+ (* src-frame channels) c)))
                           (b (aref samples-arr (+ (* next-frame channels) c)))
                           (v (+ a (floor (* (- b a) frac) 1000))))
                      (push (clamp-i16 v) out)))))
      (make-audio-buffer :sample-rate dst-rate :channels channels
                          :samples (nreverse out)))))

(defun normalize-to-peak (buf target-amplitude)
  (cond
    ((null (audio-buffer-samples buf))
     (make-audio-buffer :sample-rate (audio-buffer-sample-rate buf)
                         :channels (audio-buffer-channels buf)
                         :samples nil))
    (t
     (let ((max-abs 0))
       (dolist (s (audio-buffer-samples buf))
         (let ((a (abs s)))
           (when (> a max-abs) (setf max-abs a))))
       (cond
         ((zerop max-abs)
          (make-audio-buffer :sample-rate (audio-buffer-sample-rate buf)
                              :channels (audio-buffer-channels buf)
                              :samples (copy-list (audio-buffer-samples buf))))
         (t
          (let* ((target (max 0 (min 32767 target-amplitude)))
                 (scaled (mapcar (lambda (s)
                                    (clamp-i16 (floor (* s target) max-abs)))
                                  (audio-buffer-samples buf))))
            (make-audio-buffer :sample-rate (audio-buffer-sample-rate buf)
                                :channels (audio-buffer-channels buf)
                                :samples scaled))))))))

(defun to-mono (buf)
  (if (= (audio-buffer-channels buf) 1)
      (make-audio-buffer :sample-rate (audio-buffer-sample-rate buf)
                          :channels 1
                          :samples (copy-list (audio-buffer-samples buf)))
      (let ((samples-arr (coerce (audio-buffer-samples buf) 'vector))
            (n (audio-buffer-frame-count buf))
            (out nil))
        (loop for i from 0 below n do
              (let ((l (aref samples-arr (* i 2)))
                    (r (aref samples-arr (1+ (* i 2)))))
                (push (floor (+ l r) 2) out)))
        (make-audio-buffer :sample-rate (audio-buffer-sample-rate buf)
                            :channels 1 :samples (nreverse out)))))

(defun to-stereo (buf)
  (if (= (audio-buffer-channels buf) 2)
      (make-audio-buffer :sample-rate (audio-buffer-sample-rate buf)
                          :channels 2
                          :samples (copy-list (audio-buffer-samples buf)))
      (let ((out nil))
        (dolist (s (audio-buffer-samples buf))
          (push s out)
          (push s out))
        (make-audio-buffer :sample-rate (audio-buffer-sample-rate buf)
                            :channels 2 :samples (nreverse out)))))

;; Little-endian byte helpers
(defun put-le (out n nbytes)
  (loop for i from 0 below nbytes do
        (vector-push-extend (logand (ash n (* -8 i)) 255) out)))

(defun put-bytes (out bytes)
  (loop for b across bytes do (vector-push-extend b out)))

(defun i16-to-u16 (n)
  (if (< n 0) (+ n 65536) n))

(defun encode-wav (buf)
  (let* ((bits-per-sample 16)
         (channels (audio-buffer-channels buf))
         (sample-rate (audio-buffer-sample-rate buf))
         (byte-rate (* sample-rate channels (floor bits-per-sample 8)))
         (block-align (* channels (floor bits-per-sample 8)))
         (data-size (* (length (audio-buffer-samples buf)) 2))
         (riff-size (+ 36 data-size))
         (out (make-array 0 :element-type '(unsigned-byte 8)
                            :adjustable t :fill-pointer 0)))
    (put-bytes out (map 'vector #'char-code "RIFF"))
    (put-le out riff-size 4)
    (put-bytes out (map 'vector #'char-code "WAVE"))
    (put-bytes out (map 'vector #'char-code "fmt "))
    (put-le out 16 4)
    (put-le out 1 2)
    (put-le out channels 2)
    (put-le out sample-rate 4)
    (put-le out byte-rate 4)
    (put-le out block-align 2)
    (put-le out bits-per-sample 2)
    (put-bytes out (map 'vector #'char-code "data"))
    (put-le out data-size 4)
    (dolist (s (audio-buffer-samples buf))
      (put-le out (i16-to-u16 s) 2))
    out))

(defun read-le (bytes offset nbytes)
  (let ((v 0))
    (loop for i from 0 below nbytes do
          (incf v (ash (aref bytes (+ offset i)) (* 8 i))))
    v))

(defun read-le-signed-16 (bytes offset)
  (let ((v (read-le bytes offset 2)))
    (if (>= v 32768) (- v 65536) v)))

(defun bytes-equal-p (bytes offset str)
  (loop for i from 0 below (length str)
        always (= (aref bytes (+ offset i)) (char-code (aref str i)))))

(defun decode-wav (bytes)
  (when (< (length bytes) 44) (signal-codec-error :empty-buffer))
  (unless (and (bytes-equal-p bytes 0 "RIFF")
               (bytes-equal-p bytes 8 "WAVE"))
    (signal-codec-error :empty-buffer))
  (let* ((channels (read-le bytes 22 2))
         (sample-rate (read-le bytes 24 4))
         (data-size (read-le bytes 40 4))
         (sample-count (floor data-size 2))
         (samples (loop for i from 0 below sample-count
                          collect (read-le-signed-16 bytes (+ 44 (* i 2))))))
    (make-audio-buffer :sample-rate sample-rate :channels channels
                        :samples samples)))

(defun demo ()
  (let* ((frames (list (make-mp3-frame :sample-rate 22050 :channels 2
                                         :bitrate-kbps 128 :samples-per-frame 4
                                         :pcm '(100 -100 200 -200 400 -400 800 -800))
                        (make-mp3-frame :sample-rate 22050 :channels 2
                                         :bitrate-kbps 128 :samples-per-frame 2
                                         :pcm '(-500 500 -1000 1000))))
         (buf (decode-mp3-to-buffer frames)))
    (format t "decoded: rate=~D ch=~D samples=~D~%"
            (audio-buffer-sample-rate buf)
            (audio-buffer-channels buf)
            (length (audio-buffer-samples buf)))

    (let ((resampled (resample buf 44100)))
      (format t "resampled: rate=~D ch=~D samples=~D~%"
              (audio-buffer-sample-rate resampled)
              (audio-buffer-channels resampled)
              (length (audio-buffer-samples resampled)))

      (let ((normalized (normalize-to-peak resampled 32000)))
        (let ((max-abs 0))
          (dolist (s (audio-buffer-samples normalized))
            (let ((a (abs s))) (when (> a max-abs) (setf max-abs a))))
          (format t "normalized max_abs: ~D~%" max-abs))

        (let ((mono (to-mono normalized)))
          (format t "mono: ch=~D samples=~D~%"
                  (audio-buffer-channels mono)
                  (length (audio-buffer-samples mono)))

          (let ((wav (encode-wav mono)))
            (format t "wav bytes: ~D~%" (length wav))
            (format t "header: ~A ~A~%"
                    (map 'string #'code-char (subseq wav 0 4))
                    (map 'string #'code-char (subseq wav 8 12)))
            (let ((decoded (decode-wav wav)))
              (format t "decoded wav: rate=~D ch=~D samples=~D~%"
                      (audio-buffer-sample-rate decoded)
                      (audio-buffer-channels decoded)
                      (length (audio-buffer-samples decoded)))
              (format t "samples match: ~A~%"
                      (equal (audio-buffer-samples decoded)
                              (audio-buffer-samples mono))))))))))
