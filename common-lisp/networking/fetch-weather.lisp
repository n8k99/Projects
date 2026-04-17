;;;; Fetch Current Weather — HTTP client for live weather data from external APIs.

(defpackage #:fetch-weather
  (:use #:cl)
  (:export #:make-weather-client
           #:by-city
           #:by-coords
           #:by-zip
           #:compare-weather
           #:weather-report-city
           #:weather-report-country
           #:weather-report-latitude
           #:weather-report-longitude
           #:weather-report-temperature-k
           #:weather-report-feels-like-k
           #:weather-report-humidity
           #:weather-report-pressure-hpa
           #:weather-report-visibility-m
           #:weather-report-conditions
           #:weather-report-wind
           #:weather-report-clouds-pct
           #:weather-report-timestamp
           #:weather-report-sunrise
           #:weather-report-sunset
           #:temperature-c
           #:temperature-f
           #:feels-like-c
           #:feels-like-f
           #:wind-cardinal
           #:summary))

(in-package #:fetch-weather)

;;; --- Data Structures ---

(defstruct wind-info
  (speed-mps 0.0 :type float)
  (direction-deg 0.0 :type float)
  (gust-mps nil))

(defstruct weather-condition
  (id 0 :type integer)
  (main "" :type string)
  (description "" :type string)
  (icon "" :type string))

(defstruct weather-report
  (city "" :type string)
  (country "" :type string)
  (latitude 0.0 :type float)
  (longitude 0.0 :type float)
  (temperature-k 0.0 :type float)
  (feels-like-k 0.0 :type float)
  (humidity 0 :type integer)
  (pressure-hpa 0 :type integer)
  (visibility-m 0 :type integer)
  (conditions nil :type list)
  (wind (make-wind-info) :type wind-info)
  (clouds-pct 0 :type integer)
  (timestamp 0 :type integer)
  (sunrise nil)
  (sunset nil))

(defstruct weather-client
  (api-key "" :type string)
  (base-url "https://api.openweathermap.org" :type string))

;;; --- Temperature conversions ---

(defun temperature-c (report)
  "Convert temperature from Kelvin to Celsius."
  (- (weather-report-temperature-k report) 273.15))

(defun temperature-f (report)
  "Convert temperature from Kelvin to Fahrenheit."
  (+ (* (temperature-c report) 9/5) 32))

(defun feels-like-c (report)
  "Convert feels-like from Kelvin to Celsius."
  (- (weather-report-feels-like-k report) 273.15))

(defun feels-like-f (report)
  "Convert feels-like from Kelvin to Fahrenheit."
  (+ (* (feels-like-c report) 9/5) 32))

;;; --- Wind ---

(defun wind-speed-mph (wind)
  (* (wind-info-speed-mps wind) 2.23694))

(defun wind-speed-kmh (wind)
  (* (wind-info-speed-mps wind) 3.6))

(defparameter *cardinal-directions*
  #("N" "NNE" "NE" "ENE" "E" "ESE" "SE" "SSE"
    "S" "SSW" "SW" "WSW" "W" "WNW" "NW" "NNW"))

(defun wind-cardinal (wind)
  "Return compass direction as a string."
  (let ((idx (mod (round (/ (wind-info-direction-deg wind) 22.5)) 16)))
    (aref *cardinal-directions* idx)))

;;; --- Summary ---

(defun %description (report)
  (let ((conds (weather-report-conditions report)))
    (if conds
        (weather-condition-description (first conds))
        "unknown")))

(defun summary (report)
  "Return a one-line weather summary string."
  (let ((w (weather-report-wind report)))
    (format nil "~A, ~A: ~,1F°C (~,1F°F), ~A, humidity ~D%, wind ~,1F m/s ~A"
            (weather-report-city report)
            (weather-report-country report)
            (temperature-c report)
            (temperature-f report)
            (%description report)
            (weather-report-humidity report)
            (wind-info-speed-mps w)
            (wind-cardinal w))))

;;; --- Minimal JSON extraction (no external deps) ---

(defun %find-json-value (json key)
  "Find the value for KEY (a string like \"temp\") in a JSON string.
Returns the raw value substring, or NIL."
  (let* ((search (concatenate 'string "\"" key "\""))
         (pos (search search json)))
    (when pos
      (let* ((after (subseq json (+ pos (length search))))
             (colon-pos (position #\: after)))
        (when colon-pos
          (let ((val-start (string-left-trim '(#\Space #\Tab #\Newline #\Return)
                                              (subseq after (1+ colon-pos)))))
            (cond
              ;; String value
              ((char= (char val-start 0) #\")
               (let ((end (position #\" val-start :start 1)))
                 (when end (subseq val-start 1 end))))
              ;; Numeric or other value — read to delimiter
              (t
               (let ((end (position-if (lambda (c) (member c '(#\, #\} #\] #\Space)))
                                       val-start)))
                 (if end
                     (subseq val-start 0 end)
                     val-start))))))))))

(defun %json-float (json key &optional (default 0.0))
  (let ((v (%find-json-value json key)))
    (if v (or (ignore-errors (read-from-string v)) default) default)))

(defun %json-int (json key &optional (default 0))
  (let ((v (%find-json-value json key)))
    (if v (or (ignore-errors (parse-integer v :junk-allowed t)) default) default)))

(defun %json-string (json key &optional (default ""))
  (or (%find-json-value json key) default))

;;; --- HTTP (minimal, using usocket or sb-bsd-sockets) ---
;;; For portability, we construct a raw HTTP/1.1 request over a socket.

(defun %url-encode (string)
  "Percent-encode a string for URL query parameters."
  (with-output-to-string (s)
    (loop for c across string do
      (if (or (alphanumericp c) (member c '(#\- #\_ #\. #\~)))
          (write-char c s)
          (format s "%~2,'0X" (char-code c))))))

(defun %build-query (params api-key)
  "Build a URL query string from an alist of params plus the API key."
  (format nil "~{~A~^&~}"
          (append (loop for (k . v) in params
                        collect (format nil "~A=~A" k (%url-encode v)))
                  (list (format nil "appid=~A" (%url-encode api-key))))))

(defun %parse-conditions (json)
  "Extract weather conditions from the JSON weather array."
  (let ((start (search "\"weather\"" json)))
    (when start
      (let* ((arr-start (position #\[ json :start start))
             (arr-end (position #\] json :start (or arr-start 0))))
        (when (and arr-start arr-end)
          (let ((arr (subseq json (1+ arr-start) arr-end)))
            (list (make-weather-condition
                   :id (%json-int arr "id")
                   :main (%json-string arr "main")
                   :description (%json-string arr "description")
                   :icon (%json-string arr "icon")))))))))

(defun %parse-report (json)
  "Parse an OpenWeatherMap JSON response into a weather-report."
  (make-weather-report
   :city (%json-string json "name")
   :country (%json-string json "country")
   :latitude (%json-float json "lat")
   :longitude (%json-float json "lon")
   :temperature-k (%json-float json "temp")
   :feels-like-k (%json-float json "feels_like")
   :humidity (%json-int json "humidity")
   :pressure-hpa (%json-int json "pressure")
   :visibility-m (%json-int json "visibility")
   :conditions (%parse-conditions json)
   :wind (make-wind-info
          :speed-mps (%json-float json "speed")
          :direction-deg (%json-float json "deg")
          :gust-mps (let ((v (%find-json-value json "gust")))
                      (when v (ignore-errors (read-from-string v)))))
   :clouds-pct (%json-int json "all")
   :timestamp (%json-int json "dt")
   :sunrise (let ((v (%json-int json "sunrise" -1)))
              (when (> v 0) v))
   :sunset (let ((v (%json-int json "sunset" -1)))
             (when (> v 0) v))))

;;; --- Public API ---
;;; Note: actual HTTP calls require a socket library (usocket, dexador, drakma).
;;; These functions construct the request and parse the response; callers must
;;; provide the transport or use the demo with an HTTP library loaded.

(defun by-city (client city &optional country-code)
  "Fetch weather by city name. Returns (values query-path parse-fn).
The caller must perform the HTTP GET and pass the response body to the parse fn."
  (let ((q (if country-code
              (format nil "~A,~A" city country-code)
              city)))
    (values
     (format nil "/data/2.5/weather?~A"
             (%build-query (list (cons "q" q))
                           (weather-client-api-key client)))
     #'%parse-report)))

(defun by-coords (client lat lon)
  "Fetch weather by coordinates. Returns (values query-path parse-fn)."
  (values
   (format nil "/data/2.5/weather?~A"
           (%build-query (list (cons "lat" (format nil "~F" lat))
                               (cons "lon" (format nil "~F" lon)))
                         (weather-client-api-key client)))
   #'%parse-report))

(defun by-zip (client zip-code &optional (country-code "US"))
  "Fetch weather by zip code. Returns (values query-path parse-fn)."
  (values
   (format nil "/data/2.5/weather?~A"
           (%build-query (list (cons "zip" (format nil "~A,~A" zip-code country-code)))
                         (weather-client-api-key client)))
   #'%parse-report))

;;; --- Compare ---

(defun compare-weather (reports)
  "Compare weather across locations. Returns an alist of superlatives."
  (when reports
    (list
     (cons :warmest (weather-report-city
                     (reduce (lambda (a b) (if (> (weather-report-temperature-k a)
                                                   (weather-report-temperature-k b)) a b))
                             reports)))
     (cons :coldest (weather-report-city
                     (reduce (lambda (a b) (if (< (weather-report-temperature-k a)
                                                   (weather-report-temperature-k b)) a b))
                             reports)))
     (cons :most-humid (weather-report-city
                        (reduce (lambda (a b) (if (> (weather-report-humidity a)
                                                      (weather-report-humidity b)) a b))
                                reports)))
     (cons :windiest (weather-report-city
                      (reduce (lambda (a b)
                                (if (> (wind-info-speed-mps (weather-report-wind a))
                                       (wind-info-speed-mps (weather-report-wind b))) a b))
                              reports))))))
