-- Flower Shop Ordering — build-up/commit cart with frozen prices and reversible orders.

namespace Classes

inductive OrderStatus
  | draft
  | placed
  | fulfilled
  | cancelled
  deriving Repr, DecidableEq

structure Flower where
  sku : String
  name : String
  unitPriceCents : Int
  stock : Int := 0
  deriving Repr

structure CartLine where
  sku : String
  quantity : Int
  unitPriceCents : Int              -- frozen at add time
  deriving Repr

def CartLine.lineTotalCents (l : CartLine) : Int :=
  l.quantity * l.unitPriceCents

inductive DiscountKind
  | percentageOff
  | fixedOff
  | freeShipping
  deriving Repr, DecidableEq

structure DiscountRule where
  kind : DiscountKind
  value : Int
  minSubtotalCents : Int := 0
  note : String := ""
  deriving Repr

def DiscountRule.discountCents (r : DiscountRule) (subtotal shipping : Int) : Int :=
  if subtotal < r.minSubtotalCents then 0
  else match r.kind with
    | .percentageOff => subtotal * r.value / 100
    | .fixedOff      => min r.value subtotal
    | .freeShipping  => if subtotal ≥ r.value then shipping else 0

structure Cart where
  id : String
  ownerId : String
  lines : List CartLine := []
  discountRules : List DiscountRule := []
  shippingCents : Int := 0
  taxRateBps : Int := 0
  status : OrderStatus := .draft
  deriving Repr

structure Order where
  id : Nat
  ownerId : String
  lines : List CartLine
  discountRules : List DiscountRule
  shippingCents : Int
  taxRateBps : Int
  status : OrderStatus := .placed
  deriving Repr

structure Shop where
  flowers : List Flower := []
  carts : List Cart := []
  orders : List Order := []
  nextOrderId : Nat := 1
  deriving Repr

def Shop.empty : Shop := { }

def Shop.findFlower (s : Shop) (sku : String) : Option Flower :=
  s.flowers.find? (·.sku == sku)

def Shop.findCart (s : Shop) (id : String) : Option Cart :=
  s.carts.find? (·.id == id)

def Shop.findOrder (s : Shop) (id : Nat) : Option Order :=
  s.orders.find? (·.id == id)

def Shop.addFlower (s : Shop) (f : Flower) : Except String Shop :=
  if s.flowers.any (·.sku == f.sku)
  then .error s!"duplicate flower: {f.sku}"
  else .ok { s with flowers := s.flowers ++ [f] }

def Shop.stockFlower (s : Shop) (sku : String) (quantity : Int) : Except String Shop := do
  if quantity < 0 then throw "non-negative quantity required"
  if (s.findFlower sku).isNone then throw s!"unknown flower: {sku}"
  pure { s with flowers := s.flowers.map fun f =>
    if f.sku == sku then { f with stock := f.stock + quantity } else f }

def Shop.newCart (s : Shop) (cartId ownerId : String)
    (shippingCents : Int := 0) (taxRateBps : Int := 0) : Except String Shop :=
  if s.carts.any (·.id == cartId)
  then .error s!"duplicate cart: {cartId}"
  else .ok { s with carts := s.carts ++ [{
    id := cartId, ownerId, shippingCents, taxRateBps,
    lines := [], discountRules := [], status := .draft
  }] }

private def sumQuantityInCart (cart : Cart) (sku : String) : Int :=
  cart.lines.foldl (fun acc l => if l.sku == sku then acc + l.quantity else acc) 0

def Shop.addToCart (s : Shop) (cartId sku : String) (quantity : Int)
    : Except String Shop := do
  if quantity ≤ 0 then throw "quantity must be positive"
  let flower ← (s.findFlower sku).elim (throw s!"unknown flower: {sku}") pure
  let cart ← (s.findCart cartId).elim (throw s!"unknown cart: {cartId}") pure
  if cart.status ≠ .draft then throw s!"cart {cartId} is not draft"
  let already := sumQuantityInCart cart sku
  if flower.stock < already + quantity then
    throw s!"out of stock: {sku}"
  let newLines :=
    if cart.lines.any (·.sku == sku) then
      cart.lines.map fun l =>
        if l.sku == sku then { l with quantity := l.quantity + quantity } else l
    else
      cart.lines ++ [{ sku, quantity, unitPriceCents := flower.unitPriceCents }]
  pure { s with carts := s.carts.map fun c =>
    if c.id == cartId then { c with lines := newLines } else c }

def subtotalCents (lines : List CartLine) : Int :=
  lines.foldl (fun acc l => acc + l.lineTotalCents) 0

def discountsFor (rules : List DiscountRule) (subtotal shipping : Int) : Int :=
  rules.foldl (fun acc r => acc + r.discountCents subtotal shipping) 0

def cartTotalCents (cart : Cart) : Int :=
  let sub := subtotalCents cart.lines
  let disc := discountsFor cart.discountRules sub cart.shippingCents
  let taxable := max 0 (sub - disc)
  let tax := taxable * cart.taxRateBps / 10000
  max 0 (sub - disc + tax + cart.shippingCents)

def orderTotalCents (o : Order) : Int :=
  let sub := subtotalCents o.lines
  let disc := discountsFor o.discountRules sub o.shippingCents
  let taxable := max 0 (sub - disc)
  let tax := taxable * o.taxRateBps / 10000
  max 0 (sub - disc + tax + o.shippingCents)

def Shop.checkout (s : Shop) (cartId : String) : Except String (Shop × Order) := do
  let cart ← (s.findCart cartId).elim (throw s!"unknown cart: {cartId}") pure
  if cart.status ≠ .draft then throw s!"cart {cartId} is not draft"
  if cart.lines.isEmpty then throw s!"cart {cartId} is empty"
  -- Pre-flight.
  for l in cart.lines do
    match s.findFlower l.sku with
    | none => throw s!"unknown flower: {l.sku}"
    | some f => if f.stock < l.quantity then throw s!"out of stock: {l.sku}"
  -- Commit: decrement stock, create order, mark cart placed.
  let flowersAfter := s.flowers.map fun f =>
    match cart.lines.find? (·.sku == f.sku) with
    | some l => { f with stock := f.stock - l.quantity }
    | none => f
  let o : Order := {
    id := s.nextOrderId,
    ownerId := cart.ownerId,
    lines := cart.lines, discountRules := cart.discountRules,
    shippingCents := cart.shippingCents, taxRateBps := cart.taxRateBps,
    status := .placed
  }
  let cartsAfter := s.carts.map fun c =>
    if c.id == cartId then { c with status := .placed } else c
  pure ({
    s with flowers := flowersAfter, carts := cartsAfter,
    orders := s.orders ++ [o], nextOrderId := s.nextOrderId + 1
  }, o)

def Shop.cancelOrder (s : Shop) (orderId : Nat) : Except String Shop := do
  let o ← (s.findOrder orderId).elim (throw s!"unknown order: {orderId}") pure
  if o.status ≠ .placed then throw s!"order {orderId} is not placed"
  -- Reverse inventory.
  let flowersAfter := s.flowers.map fun f =>
    match o.lines.find? (·.sku == f.sku) with
    | some l => { f with stock := f.stock + l.quantity }
    | none => f
  let ordersAfter := s.orders.map fun x =>
    if x.id == orderId then { x with status := .cancelled } else x
  pure { s with flowers := flowersAfter, orders := ordersAfter }

end Classes
