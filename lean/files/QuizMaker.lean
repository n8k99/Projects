-- Quiz Maker — polymorphic questions with file round-trip serialisation.

namespace Files.QM

inductive Kind where | mc | tf | sa
  deriving Repr, DecidableEq, BEq

structure Question where
  kind : Kind
  prompt : String
  options : List String := []     -- MC
  correctIndex : Nat := 0          -- MC
  correctBool : Bool := false      -- TF
  expected : String := ""          -- SA
  deriving Repr

inductive Answer where
  | mc (choice : Nat)
  | tf (truth : Bool)
  | sa (text : String)
  deriving Repr

structure Quiz where
  id : Nat
  title : String
  questions : List Question := []
  deriving Repr

structure Score where
  total : Nat
  correct : Nat
  skipped : Nat
  wrong : Nat
  deriving Repr

def Score.percent (s : Score) : Float :=
  if s.total == 0 then 0.0
  else s.correct.toFloat / s.total.toFloat * 100.0

def checkAnswer (q : Question) (a : Answer) : Bool :=
  match q.kind, a with
  | .mc, .mc i => i == q.correctIndex
  | .tf, .tf b => b == q.correctBool
  | .sa, .sa s => q.expected.trim.toLower == s.trim.toLower
  | _, _ => false

def scoreAttempt (q : Quiz) (answers : List (Option Answer)) : Score := Id.run do
  let mut correct := 0
  let mut skipped := 0
  let mut wrong := 0
  for i in [0:q.questions.length] do
    let qn := q.questions.get! i
    let a := answers.get? i |>.bind id
    match a with
    | none => skipped := skipped + 1
    | some ans =>
      if checkAnswer qn ans then correct := correct + 1
      else wrong := wrong + 1
  return { total := q.questions.length, correct, skipped, wrong }

def Quiz.toText (q : Quiz) : String := Id.run do
  let mut out := s!"QUIZ\nid: {q.id}\ntitle: {q.title}\n"
  for qn in q.questions do
    out := out ++ "---\n"
    match qn.kind with
    | .mc =>
      out := out ++ s!"kind: mc\nprompt: {qn.prompt}\n"
      for o in qn.options do
        out := out ++ s!"option: {o}\n"
      out := out ++ s!"correct: {qn.correctIndex}\n"
    | .tf =>
      out := out ++ s!"kind: tf\nprompt: {qn.prompt}\ncorrect: {qn.correctBool}\n"
    | .sa =>
      out := out ++ s!"kind: sa\nprompt: {qn.prompt}\nexpected: {qn.expected}\n"
  return out

end Files.QM
