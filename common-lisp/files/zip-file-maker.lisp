;;;; Zip File Maker — archive multiple named files with per-entry RLE compression.

(defpackage #:zip-file-maker
  (:use #:cl)
  (:export #:make-archive #:add-file #:add-stored #:extract
           #:names #:compressed-size #:compression-ratio
           #:rle-encode #:rle-decode))

(in-package #:zip-file-maker)

(defstruct entry
  (name "" :type string)
  (method :store :type keyword)
  (size-original 0 :type integer)
  (data #() :type (simple-array (unsigned-byte 8) (*))))

(defstruct archive
  (entries nil :type list))

(defun make-bytes (&rest vals)
  (make-array (length vals)
              :element-type '(unsigned-byte 8)
              :initial-contents vals))

(defun bytes-concat (&rest byte-arrays)
  (let* ((total (reduce #'+ byte-arrays :key #'length))
         (out (make-array total :element-type '(unsigned-byte 8))))
    (loop with i = 0
          for arr in byte-arrays
          do (loop for b across arr do
                   (setf (aref out i) b)
                   (incf i)))
    out))

(defun rle-encode (data)
  (let ((out (make-array 0 :element-type '(unsigned-byte 8)
                           :adjustable t :fill-pointer 0))
        (i 0)
        (n (length data)))
    (loop while (< i n) do
          (let ((byte (aref data i))
                (run 1))
            (loop while (and (< (+ i run) n)
                             (= (aref data (+ i run)) byte)
                             (< run 255))
                  do (incf run))
            (vector-push-extend run out)
            (vector-push-extend byte out)
            (incf i run)))
    (coerce out '(simple-array (unsigned-byte 8) (*)))))

(defun rle-decode (encoded)
  (let ((out (make-array 0 :element-type '(unsigned-byte 8)
                           :adjustable t :fill-pointer 0))
        (i 0)
        (n (length encoded)))
    (loop while (< (1+ i) n) do
          (let ((count (aref encoded i))
                (byte (aref encoded (1+ i))))
            (dotimes (j count) (vector-push-extend byte out))
            (incf i 2)))
    (coerce out '(simple-array (unsigned-byte 8) (*)))))

(defun to-bytes (seq)
  (coerce seq '(simple-array (unsigned-byte 8) (*))))

(defun add-file (archive name data)
  (let* ((data (to-bytes data))
         (rle (rle-encode data))
         (use-rle (< (length rle) (length data))))
    (setf (archive-entries archive)
          (append (archive-entries archive)
                  (list (make-entry :name name
                                     :method (if use-rle :rle :store)
                                     :size-original (length data)
                                     :data (if use-rle rle data)))))))

(defun add-stored (archive name data)
  (let ((data (to-bytes data)))
    (setf (archive-entries archive)
          (append (archive-entries archive)
                  (list (make-entry :name name
                                     :method :store
                                     :size-original (length data)
                                     :data data))))))

(defun extract (archive name)
  (let ((entry (find-if (lambda (e) (string= (entry-name e) name))
                         (archive-entries archive))))
    (when entry
      (case (entry-method entry)
        (:store (entry-data entry))
        (:rle (rle-decode (entry-data entry)))))))

(defun names (archive)
  (mapcar #'entry-name (archive-entries archive)))

(defun compressed-size (archive)
  (reduce #'+ (archive-entries archive) :key (lambda (e) (length (entry-data e)))
          :initial-value 0))

(defun compression-ratio (archive)
  (let ((orig (reduce #'+ (archive-entries archive) :key #'entry-size-original
                      :initial-value 0)))
    (if (zerop orig) 1.0 (/ (compressed-size archive) (coerce orig 'single-float)))))

(defun string-to-bytes (s)
  (map '(simple-array (unsigned-byte 8) (*)) #'char-code s))

(defun bytes-to-string (b)
  (map 'string #'code-char b))

(defun demo ()
  (let ((a (make-archive)))
    (add-file a "repetitive.txt" (string-to-bytes "aaaaaaaaaa"))
    (add-file a "random.bin" (make-bytes 1 2 3 4 5))
    (add-file a "big.txt" (string-to-bytes (make-string 1000 :initial-element #\x)))
    (add-file a "mixed.txt" (string-to-bytes "the quick brown fox aaaaaaa xyz"))
    (format t "names: ~A~%" (names a))
    (dolist (e (archive-entries a))
      (format t "  ~A  method=~A  orig=~D  stored=~D~%"
              (entry-name e) (entry-method e)
              (entry-size-original e) (length (entry-data e))))
    (format t "~%total compressed: ~D bytes~%" (compressed-size a))
    (format t "ratio: ~,4F~%" (compression-ratio a))
    (let ((rep (extract a "repetitive.txt"))
          (big (extract a "big.txt")))
      (format t "~%round-trip repetitive.txt: ~D bytes, all x: ~A~%"
              (length rep)
              (every (lambda (b) (= b (char-code #\a))) rep))
      (format t "round-trip big.txt: ~D bytes~%" (length big)))))
