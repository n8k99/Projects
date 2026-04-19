// Flower Shop Ordering — build-up/commit cart with frozen prices and reversible orders.
package classes

import (
	"errors"
	"fmt"
	"time"
)

// ShopOrderStatus enumerates an order's lifecycle.
type ShopOrderStatus string

const (
	OrderDraft     ShopOrderStatus = "draft"
	OrderPlaced    ShopOrderStatus = "placed"
	OrderFulfilled ShopOrderStatus = "fulfilled"
	OrderCancelled ShopOrderStatus = "cancelled"
)

// Flower is a sellable catalog item.
type Flower struct {
	SKU            string
	Name           string
	UnitPriceCents int64
	Stock          int64
}

// CartLine is one line in a cart. UnitPriceCents is frozen at add-time.
type CartLine struct {
	SKU            string
	Quantity       int64
	UnitPriceCents int64
}

// LineTotalCents returns Quantity * UnitPriceCents.
func (l CartLine) LineTotalCents() int64 { return l.Quantity * l.UnitPriceCents }

// DiscountKind enumerates the closed set of rule types.
type DiscountKind string

const (
	DiscountPercentageOff DiscountKind = "percentage_off"
	DiscountFixedOff      DiscountKind = "fixed_off"
	DiscountFreeShipping  DiscountKind = "free_shipping"
)

// DiscountRule captures one applicable discount.
type DiscountRule struct {
	Kind             DiscountKind
	Value            int64
	MinSubtotalCents int64
	Note             string
}

// DiscountCents returns the discount amount given a subtotal and shipping.
func (d DiscountRule) DiscountCents(subtotal, shipping int64) int64 {
	if subtotal < d.MinSubtotalCents {
		return 0
	}
	switch d.Kind {
	case DiscountPercentageOff:
		return subtotal * d.Value / 100
	case DiscountFixedOff:
		if d.Value < subtotal {
			return d.Value
		}
		return subtotal
	case DiscountFreeShipping:
		if subtotal >= d.Value {
			return shipping
		}
	}
	return 0
}

// Cart is a mutable draft.
type Cart struct {
	ID             string
	OwnerID        string
	Lines          []CartLine
	DiscountRules  []DiscountRule
	ShippingCents  int64
	TaxRateBps     int64
	Status         ShopOrderStatus
}

// ShopOrder is a committed cart.
type ShopOrder struct {
	ID             uint64
	OwnerID        string
	Lines          []CartLine
	DiscountRules  []DiscountRule
	ShippingCents  int64
	TaxRateBps     int64
	PlacedAt       time.Time
	Status         ShopOrderStatus
}

// Shop errors.
var (
	ErrUnknownFlower      = errors.New("unknown flower")
	ErrUnknownCart        = errors.New("unknown cart")
	ErrUnknownOrder       = errors.New("unknown order")
	ErrDuplicateFlower    = errors.New("duplicate flower")
	ErrDuplicateCart      = errors.New("duplicate cart")
	ErrInvalidQuantity    = errors.New("invalid quantity")
	ErrOutOfStock         = errors.New("out of stock")
	ErrBadOrderTransition = errors.New("bad order transition")
	ErrEmptyCart          = errors.New("empty cart")
	ErrNotInCart          = errors.New("not in cart")
)

// Shop coordinates flowers, carts, and orders.
type Shop struct {
	Flowers      map[string]*Flower
	Carts        map[string]*Cart
	Orders       map[uint64]*ShopOrder
	nextOrderID  uint64
}

// NewShop builds an empty shop.
func NewShop() *Shop {
	return &Shop{
		Flowers: make(map[string]*Flower),
		Carts:   make(map[string]*Cart),
		Orders:  make(map[uint64]*ShopOrder),
		nextOrderID: 1,
	}
}

// AddFlower registers a flower.
func (s *Shop) AddFlower(f Flower) error {
	if _, ok := s.Flowers[f.SKU]; ok {
		return fmt.Errorf("%w: %s", ErrDuplicateFlower, f.SKU)
	}
	s.Flowers[f.SKU] = &f
	return nil
}

// Stock increases inventory for a flower.
func (s *Shop) Stock(sku string, quantity int64) error {
	if quantity < 0 {
		return ErrInvalidQuantity
	}
	f, ok := s.Flowers[sku]
	if !ok {
		return fmt.Errorf("%w: %s", ErrUnknownFlower, sku)
	}
	f.Stock += quantity
	return nil
}

// NewCart creates an empty draft cart.
func (s *Shop) NewCart(cartID, ownerID string, shippingCents, taxRateBps int64) error {
	if _, ok := s.Carts[cartID]; ok {
		return fmt.Errorf("%w: %s", ErrDuplicateCart, cartID)
	}
	s.Carts[cartID] = &Cart{
		ID: cartID, OwnerID: ownerID,
		ShippingCents: shippingCents, TaxRateBps: taxRateBps,
		Status: OrderDraft,
	}
	return nil
}

func (s *Shop) requireDraft(cartID string) (*Cart, error) {
	cart, ok := s.Carts[cartID]
	if !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownCart, cartID)
	}
	if cart.Status != OrderDraft {
		return nil, fmt.Errorf("%w: cart %s is %s", ErrBadOrderTransition, cartID, cart.Status)
	}
	return cart, nil
}

// AddToCart appends (or merges) a line, freezing the price.
func (s *Shop) AddToCart(cartID, sku string, quantity int64) error {
	if quantity <= 0 {
		return ErrInvalidQuantity
	}
	flower, ok := s.Flowers[sku]
	if !ok {
		return fmt.Errorf("%w: %s", ErrUnknownFlower, sku)
	}
	cart, err := s.requireDraft(cartID)
	if err != nil {
		return err
	}
	var already int64
	for _, l := range cart.Lines {
		if l.SKU == sku {
			already += l.Quantity
		}
	}
	if flower.Stock < already+quantity {
		return fmt.Errorf("%w: %s wants %d, stock %d", ErrOutOfStock, sku, already+quantity, flower.Stock)
	}
	for i := range cart.Lines {
		if cart.Lines[i].SKU == sku {
			cart.Lines[i].Quantity += quantity
			return nil
		}
	}
	cart.Lines = append(cart.Lines, CartLine{
		SKU: sku, Quantity: quantity, UnitPriceCents: flower.UnitPriceCents,
	})
	return nil
}

// UpdateQuantity changes or removes a line.
func (s *Shop) UpdateQuantity(cartID, sku string, newQuantity int64) error {
	if newQuantity < 0 {
		return ErrInvalidQuantity
	}
	f, ok := s.Flowers[sku]
	if !ok {
		return fmt.Errorf("%w: %s", ErrUnknownFlower, sku)
	}
	cart, err := s.requireDraft(cartID)
	if err != nil {
		return err
	}
	if newQuantity > 0 && f.Stock < newQuantity {
		return fmt.Errorf("%w: %s", ErrOutOfStock, sku)
	}
	for i, l := range cart.Lines {
		if l.SKU == sku {
			if newQuantity == 0 {
				cart.Lines = append(cart.Lines[:i], cart.Lines[i+1:]...)
			} else {
				cart.Lines[i].Quantity = newQuantity
			}
			return nil
		}
	}
	return fmt.Errorf("%w: %s", ErrNotInCart, sku)
}

// ApplyDiscount appends a discount rule to the cart.
func (s *Shop) ApplyDiscount(cartID string, rule DiscountRule) error {
	cart, err := s.requireDraft(cartID)
	if err != nil {
		return err
	}
	cart.DiscountRules = append(cart.DiscountRules, rule)
	return nil
}

// SubtotalCents sums line totals.
func SubtotalCents(lines []CartLine) int64 {
	var total int64
	for _, l := range lines {
		total += l.LineTotalCents()
	}
	return total
}

func discountsFor(rules []DiscountRule, subtotal, shipping int64) int64 {
	var total int64
	for _, r := range rules {
		total += r.DiscountCents(subtotal, shipping)
	}
	return total
}

// CartTotalCents computes the final amount due for a cart.
func CartTotalCents(cart *Cart) int64 {
	sub := SubtotalCents(cart.Lines)
	disc := discountsFor(cart.DiscountRules, sub, cart.ShippingCents)
	taxable := sub - disc
	if taxable < 0 {
		taxable = 0
	}
	tax := taxable * cart.TaxRateBps / 10_000
	total := sub - disc + tax + cart.ShippingCents
	if total < 0 {
		total = 0
	}
	return total
}

// OrderTotalCents computes the total for a placed order.
func OrderTotalCents(o *ShopOrder) int64 {
	sub := SubtotalCents(o.Lines)
	disc := discountsFor(o.DiscountRules, sub, o.ShippingCents)
	taxable := sub - disc
	if taxable < 0 {
		taxable = 0
	}
	tax := taxable * o.TaxRateBps / 10_000
	total := sub - disc + tax + o.ShippingCents
	if total < 0 {
		total = 0
	}
	return total
}

// Checkout commits the cart atomically, decrementing inventory.
func (s *Shop) Checkout(cartID string) (uint64, error) {
	cart, err := s.requireDraft(cartID)
	if err != nil {
		return 0, err
	}
	if len(cart.Lines) == 0 {
		return 0, ErrEmptyCart
	}
	// Pre-flight stock check.
	for _, l := range cart.Lines {
		f, ok := s.Flowers[l.SKU]
		if !ok {
			return 0, fmt.Errorf("%w: %s", ErrUnknownFlower, l.SKU)
		}
		if f.Stock < l.Quantity {
			return 0, fmt.Errorf("%w: %s (want %d, have %d)", ErrOutOfStock, l.SKU, l.Quantity, f.Stock)
		}
	}
	// Commit.
	for _, l := range cart.Lines {
		s.Flowers[l.SKU].Stock -= l.Quantity
	}
	id := s.nextOrderID
	s.nextOrderID++
	s.Orders[id] = &ShopOrder{
		ID: id, OwnerID: cart.OwnerID,
		Lines: append([]CartLine(nil), cart.Lines...),
		DiscountRules: append([]DiscountRule(nil), cart.DiscountRules...),
		ShippingCents: cart.ShippingCents, TaxRateBps: cart.TaxRateBps,
		PlacedAt: time.Now(), Status: OrderPlaced,
	}
	cart.Status = OrderPlaced
	return id, nil
}

// CancelOrder reverses a placed order's inventory decrement.
func (s *Shop) CancelOrder(orderID uint64) error {
	o, ok := s.Orders[orderID]
	if !ok {
		return fmt.Errorf("%w: %d", ErrUnknownOrder, orderID)
	}
	if o.Status != OrderPlaced {
		return fmt.Errorf("%w: order %d is %s", ErrBadOrderTransition, orderID, o.Status)
	}
	o.Status = OrderCancelled
	for _, l := range o.Lines {
		if f, ok := s.Flowers[l.SKU]; ok {
			f.Stock += l.Quantity
		}
	}
	return nil
}

// FulfillOrder marks a placed order fulfilled.
func (s *Shop) FulfillOrder(orderID uint64) error {
	o, ok := s.Orders[orderID]
	if !ok {
		return fmt.Errorf("%w: %d", ErrUnknownOrder, orderID)
	}
	if o.Status != OrderPlaced {
		return fmt.Errorf("%w: order %d is %s", ErrBadOrderTransition, orderID, o.Status)
	}
	o.Status = OrderFulfilled
	return nil
}
