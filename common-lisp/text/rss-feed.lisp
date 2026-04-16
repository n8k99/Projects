;;;; RSS Feed Creator — generate and parse RSS 2.0 XML feeds.

(defpackage :text/rss-feed
  (:use :cl)
  (:export #:rss-item
           #:make-rss-item
           #:rss-item-title
           #:rss-item-link
           #:rss-item-description
           #:rss-item-pub-date
           #:rss-item-guid
           #:rss-feed
           #:make-rss-feed
           #:rss-feed-title
           #:rss-feed-link
           #:rss-feed-description
           #:rss-feed-language
           #:rss-feed-items
           #:add-item
           #:to-xml
           #:from-xml))

(in-package :text/rss-feed)

;;; --- Data structures ---

(defstruct rss-item
  (title "" :type string)
  (link "" :type string)
  (description "" :type string)
  (pub-date "" :type string)
  (guid "" :type string))

(defstruct rss-feed
  (title "" :type string)
  (link "" :type string)
  (description "" :type string)
  (language "en-us" :type string)
  (items nil :type list))

;;; --- XML helpers ---

(defun xml-escape (str)
  "Escape special XML characters in STR."
  (let ((result str))
    (setf result (replace-all result "&" "&amp;"))
    (setf result (replace-all result "<" "&lt;"))
    (setf result (replace-all result ">" "&gt;"))
    (setf result (replace-all result "\"" "&quot;"))
    (setf result (replace-all result "'" "&apos;"))
    result))

(defun xml-unescape (str)
  "Unescape XML entities in STR."
  (let ((result str))
    (setf result (replace-all result "&lt;" "<"))
    (setf result (replace-all result "&gt;" ">"))
    (setf result (replace-all result "&quot;" "\""))
    (setf result (replace-all result "&apos;" "'"))
    (setf result (replace-all result "&amp;" "&"))
    result))

(defun replace-all (string old new)
  "Replace all occurrences of OLD with NEW in STRING."
  (let ((result (make-string-output-stream))
        (old-len (length old))
        (pos 0))
    (loop
      (let ((found (search old string :start2 pos)))
        (if found
            (progn
              (write-string (subseq string pos found) result)
              (write-string new result)
              (setf pos (+ found old-len)))
            (progn
              (write-string (subseq string pos) result)
              (return)))))
    (get-output-stream-string result)))

(defun wrap-tag (tag content &optional (indent 0))
  "Wrap CONTENT in an XML tag with given indentation."
  (let ((pad (make-string indent :initial-element #\Space)))
    (format nil "~A<~A>~A</~A>" pad tag (xml-escape content) tag)))

(defun extract-tag-content (xml tag)
  "Extract text between <TAG> and </TAG> in XML. Returns NIL if not found."
  (let* ((open (format nil "<~A>" tag))
         (close (format nil "</~A>" tag))
         (start (search open xml)))
    (when start
      (let* ((content-start (+ start (length open)))
             (end (search close xml :start2 content-start)))
        (when end
          (xml-unescape (subseq xml content-start end)))))))

(defun extract-all-between (xml open-tag close-tag)
  "Extract all substrings between OPEN-TAG and CLOSE-TAG."
  (let ((results nil)
        (pos 0))
    (loop
      (let ((start (search open-tag xml :start2 pos)))
        (unless start (return (nreverse results)))
        (let ((end (search close-tag xml :start2 (+ start (length open-tag)))))
          (unless end (return (nreverse results)))
          (push (subseq xml (+ start (length open-tag)) end) results)
          (setf pos (+ end (length close-tag))))))))

;;; --- Public API ---

(defun add-item (feed title link description &key (pub-date "") (guid ""))
  "Add an item to FEED. Returns the modified feed."
  (let ((actual-guid (if (string= guid "") link guid)))
    (push (make-rss-item :title title
                         :link link
                         :description description
                         :pub-date pub-date
                         :guid actual-guid)
          (rss-feed-items feed)))
  feed)

(defun to-xml (feed)
  "Render FEED as a valid RSS 2.0 XML string."
  (with-output-to-string (out)
    (format out "<?xml version=\"1.0\" encoding=\"UTF-8\"?>~%")
    (format out "<rss version=\"2.0\">~%")
    (format out "  <channel>~%")
    (format out "    ~A~%" (wrap-tag "title" (rss-feed-title feed)))
    (format out "    ~A~%" (wrap-tag "link" (rss-feed-link feed)))
    (format out "    ~A~%" (wrap-tag "description" (rss-feed-description feed)))
    (format out "    ~A~%" (wrap-tag "language" (rss-feed-language feed)))
    ;; Items are pushed in reverse order, so reverse for output
    (dolist (item (reverse (rss-feed-items feed)))
      (format out "    <item>~%")
      (format out "      ~A~%" (wrap-tag "title" (rss-item-title item)))
      (format out "      ~A~%" (wrap-tag "link" (rss-item-link item)))
      (format out "      ~A~%" (wrap-tag "description" (rss-item-description item)))
      (when (plusp (length (rss-item-pub-date item)))
        (format out "      ~A~%" (wrap-tag "pubDate" (rss-item-pub-date item))))
      (when (plusp (length (rss-item-guid item)))
        (format out "      ~A~%" (wrap-tag "guid" (rss-item-guid item))))
      (format out "    </item>~%"))
    (format out "  </channel>~%")
    (format out "</rss>")))

(defun from-xml (xml-str)
  "Parse an RSS 2.0 XML string back into an rss-feed struct."
  (let* ((channel-start (search "<channel>" xml-str))
         (channel-end (search "</channel>" xml-str))
         (channel (subseq xml-str (+ channel-start 9) channel-end))
         ;; Get channel-level metadata (before first <item>)
         (first-item (search "<item>" channel))
         (meta (if first-item (subseq channel 0 first-item) channel))
         (feed (make-rss-feed
                :title (or (extract-tag-content meta "title") "")
                :link (or (extract-tag-content meta "link") "")
                :description (or (extract-tag-content meta "description") "")
                :language (or (extract-tag-content meta "language") "en-us"))))
    ;; Parse items
    (let ((item-bodies (extract-all-between channel "<item>" "</item>")))
      (dolist (body (reverse item-bodies))
        (push (make-rss-item
               :title (or (extract-tag-content body "title") "")
               :link (or (extract-tag-content body "link") "")
               :description (or (extract-tag-content body "description") "")
               :pub-date (or (extract-tag-content body "pubDate") "")
               :guid (or (extract-tag-content body "guid") ""))
              (rss-feed-items feed))))
    feed))
