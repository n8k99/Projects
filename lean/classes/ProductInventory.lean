-- Product Inventory — entities with identity, movements as a ledger, current quantity as aggregate.

namespace Classes

structure Product where
  sku : String
  name : String
  price : Float
  category : String := ""
  reorderPoint : Int := 0
  deriving Repr

inductive MovementKind
  | receive
  | sell
  deriving Repr, DecidableEq

structure Movement where
  sku : String
  kind : MovementKind
  quantity : Int
  note : String := ""
  deriving Repr

def Movement.signed (m : Movement) : Int :=
  match m.kind with
  | .receive => m.quantity
  | .sell    => -m.quantity

structure Inventory where
  products : List Product := []
  movements : List Movement := []
  deriving Repr

def Inventory.empty : Inventory := { }

def Inventory.get (inv : Inventory) (sku : String) : Option Product :=
  inv.products.find? (·.sku == sku)

def Inventory.addProduct (inv : Inventory) (p : Product) : Except String Inventory :=
  if inv.products.any (·.sku == p.sku)
  then .error s!"duplicate sku: {p.sku}"
  else .ok { inv with products := inv.products ++ [p] }

def Inventory.quantity (inv : Inventory) (sku : String) : Int :=
  inv.movements.foldl (fun acc m => if m.sku == sku then acc + m.signed else acc) 0

def Inventory.receive (inv : Inventory) (sku : String) (qty : Int) (note : String := "")
    : Except String Inventory :=
  if (inv.get sku).isNone then
    .error s!"unknown sku: {sku}"
  else if qty ≤ 0 then
    .error "receive quantity must be positive"
  else
    .ok { inv with movements := inv.movements ++ [{ sku, kind := .receive, quantity := qty, note }] }

def Inventory.sell (inv : Inventory) (sku : String) (qty : Int) (note : String := "")
    : Except String Inventory :=
  if (inv.get sku).isNone then
    .error s!"unknown sku: {sku}"
  else if qty ≤ 0 then
    .error "sell quantity must be positive"
  else
    let onHand := inv.quantity sku
    if onHand < qty then
      .error s!"insufficient stock for {sku}: on hand {onHand}, requested {qty}"
    else
      .ok { inv with movements := inv.movements ++ [{ sku, kind := .sell, quantity := qty, note }] }

def Inventory.history (inv : Inventory) (sku : String) : List Movement :=
  inv.movements.filter (·.sku == sku)

def Inventory.totalValue (inv : Inventory) : Float :=
  inv.products.foldl (fun acc p => acc + p.price * (inv.quantity p.sku).toFloat) 0.0

def Inventory.restockNeeded (inv : Inventory) : List Product :=
  inv.products.filter fun p =>
    p.reorderPoint > 0 && inv.quantity p.sku ≤ p.reorderPoint

def Inventory.byCategory (inv : Inventory) (cat : String) : List Product :=
  inv.products.filter (·.category == cat)

end Classes
