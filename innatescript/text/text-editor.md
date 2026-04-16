---
id: G021
title: Text Editor
domain: text
type: rosetta-stone
status: active
depends_on: [G017, G019]
concepts:
  - state machine
  - event sourcing
  - undo/redo
  - cursor/attention
  - command language
  - buffer model
  - collaborative editing
---

# G021 — Text Editor

A text buffer with cursor and edit operations — the model behind any text editor.

## Insight: The First Stateful Object

Numbers were stateless functions. The alarm clock's state was implicit. The text editor has **explicit, mutable state**: a buffer, a cursor position, an undo stack. Each operation transforms the state. This is the Rosetta Stone's first confrontation with state management.

## Insight: The Undo Stack is a History

The undo stack records every edit — it's **event sourcing**. The current buffer is the result of applying all edits from the beginning. The stack IS the document's history. Replay it forward: you get the document. Replay it backward: you undo. In InnateScript, every choreography could carry an undo stack — a history of agent actions replayable in reverse.

## Insight: The Buffer is a Document

Content organized in lines, navigable by position. The vault is a collection of documents. Each vault file is a TextEditor buffer. The text editor isn't a utility — it's the **primitive the vault is made of**.

## Insight: Cursor Position is Attention

Where in the document are you focused? Agents have attention too: which part of a document are they reading or editing? The cursor is the simplest model of **agent attention** — a point of focus in structured content.

## Insight: Find/Replace is Pattern-Based Transformation

Search for structure, transform matches. This combines G019's search (palindrome detection) with G017's transformation (pig latin). Find identifies. Replace transforms. Together: a **content rewriting engine**.

## Insight: Editor Operations are a Command Language

Insert, delete, move, undo — a DSL for manipulating text. This is eerily close to what InnateScript itself is: a command language for manipulating the noosphere. The text editor's command set is a **micro-InnateScript**.

## The Shape

```innate
(define-shape text-editor
  "A buffer with cursor, edit operations, and undo history."

  ;; --- State ---
  (state
    (lines    : (vector-of string))   ; the buffer content
    (cursor   : (record line nat col nat))  ; attention point
    (history  : (stack-of edit-action)))     ; undo stack / event log

  ;; --- Edit actions (the event language) ---
  (define-enum edit-action
    (insert-char cursor char)
    (delete-char cursor char)
    (insert-line cursor)
    (delete-line cursor content index)
    (replace-all cursor old-lines))

  ;; --- Operations (the command language) ---
  (define (insert-char editor ch)
    "Insert CH at cursor. Record action. Advance cursor."
    (let* ((line   (get-current-line editor))
           (before (substring line 0 (cursor-col editor)))
           (after  (substring line (cursor-col editor))))
      (-> editor
          (push-history (make-edit :insert-char (cursor editor) ch))
          (set-current-line (concat before (string ch) after))
          (advance-cursor 1))))

  (define (delete-char editor)
    "Backspace at cursor. Record action. Retreat cursor."
    (cond
      ((and (= (cursor-col editor) 0)
            (= (cursor-line editor) 0))
       editor)  ; nothing to delete
      ((> (cursor-col editor) 0)
       (let* ((line (get-current-line editor))
              (ch   (char-at line (- (cursor-col editor) 1))))
         (-> editor
             (push-history (make-edit :delete-char (cursor editor) ch))
             (set-current-line (remove-char-at line (- (cursor-col editor) 1)))
             (retreat-cursor 1))))
      (else
       ;; Join with previous line
       (-> editor
           (push-history (make-edit :join-lines (cursor editor)
                                    (get-current-line editor)))
           (join-with-previous-line)))))

  (define (insert-line editor)
    "Split current line at cursor (Enter)."
    (let* ((line   (get-current-line editor))
           (before (substring line 0 (cursor-col editor)))
           (after  (substring line (cursor-col editor))))
      (-> editor
          (push-history (make-edit :insert-line (cursor editor)))
          (set-current-line before)
          (insert-line-after (cursor-line editor) after)
          (move-cursor-to (+ (cursor-line editor) 1) 0))))

  (define (delete-line editor)
    "Remove current line."
    (-> editor
        (push-history (make-edit :delete-line (cursor editor)
                                  (get-current-line editor)
                                  (cursor-line editor)))
        (remove-line-at (cursor-line editor))
        (clamp-cursor)))

  (define (undo editor)
    "Pop the history stack, reverse the recorded action."
    (if (empty? (history editor))
        editor
        (let ((action (peek (history editor))))
          (-> editor
              (pop-history)
              (reverse-action action)))))

  ;; --- Search ---
  (define (find-pattern editor pattern)
    "Find all occurrences of PATTERN. Returns list of (line col)."
    (flatmap (enumerate (lines editor))
             (lambda (i line)
               (map (find-all-indices line pattern)
                    (lambda (col) (list i col))))))

  (define (replace-pattern editor pattern replacement &optional (count -1))
    "Replace occurrences of PATTERN. Record full state for undo."
    (let ((old-lines (copy (lines editor))))
      (-> editor
          (push-history (make-edit :replace-all (cursor editor) old-lines))
          (transform-lines
            (lambda (line)
              (string-replace line pattern replacement count)))))))
```

## Choreographic Extension: Collaborative Editing

The single-cursor editor becomes profound when multiple agents share a buffer:

```innate
(define-choreography collaborative-edit
  "Multiple agents editing a shared buffer with conflict resolution."

  (roles sylvia lena vincent)

  ;; Each agent has their own cursor (attention point)
  ;; The buffer is shared, but views may differ

  (phase :draft
    (parallel
      (sylvia -> buffer (insert-text "rough draft content"))
      (lena   -> buffer (annotate "structural notes")))
    ;; Conflict: both wrote to same region
    ;; Resolution: operational transform — Lena's annotations
    ;; shift to accommodate Sylvia's insertions)

  (phase :refine
    (sequential
      (lena   -> buffer (delete-redundancy))   ; remove excess
      (vincent -> buffer (adjust-formatting))  ; normalize style
      (sylvia -> buffer (review-changes)))     ; accept/reject

  ;; The undo stack records WHO made each edit
  ;; Undo can target a specific agent's changes
  ;; This is the CRDT/OT problem — and InnateScript's
  ;; choreography model naturally expresses it as
  ;; coordinated agent actions on shared state)
```

## What This Means for InnateScript

The text editor reveals that InnateScript needs:

1. **Explicit state management** — not just function composition, but state that persists and transforms across operations. The `state` block in the shape definition.

2. **Event sourcing as a primitive** — every state change recorded as an event. The history stack isn't an add-on; it's fundamental to how state works in the noosphere.

3. **Attention modeling** — agents need cursors. When Sylvia reads a document, where is she looking? The cursor abstraction generalizes to any agent's focus point in any structured content.

4. **Command languages as shapes** — the editor's operations (insert, delete, move) form a mini-language. InnateScript itself is a command language for the noosphere. Shapes can define their own command sub-languages.

5. **Collaborative state** — when multiple agents share state, you need conflict resolution. The choreography model handles this: roles, phases, and the undo stack's attribution of edits to agents.

The text editor is where data (the buffer) meets behavior (the commands) meets history (the undo stack) meets attention (the cursor). It's the most complete model of agent-document interaction in the Rosetta Stone so far.
