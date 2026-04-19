// Product Inventory — entities with identity, movements as a ledger, current quantity as aggregate.
package classes

import (
	"errors"
	"fmt"
	"time"
)

// Product is an entity with identity (SKU) and mutable fields.
type Product struct {
	SKU           string
	Name          string
	Price         float64
	Category      string
	ReorderPoint  int
}

// MovementKind is "receive" or "sell".
type MovementKind string

const (
	MovementReceive MovementKind = "receive"
	MovementSell    MovementKind = "sell"
)

// Movement is a stock event recorded to the ledger.
type Movement struct {
	SKU      string
	Kind     MovementKind
	Quantity int
	At       time.Time
	Note     string
}

// Signed returns the movement's effect on quantity (+receive, -sell).
func (m Movement) Signed() int {
	if m.Kind == MovementReceive {
		return m.Quantity
	}
	return -m.Quantity
}

// Inventory errors.
var (
	ErrUnknownSKU         = errors.New("unknown sku")
	ErrDuplicateSKU       = errors.New("duplicate sku")
	ErrNonPositiveQty     = errors.New("quantity must be positive")
	ErrInsufficientStock  = errors.New("insufficient stock")
)

// Inventory aggregates movements to report current quantity per product.
type Inventory struct {
	Products  map[string]Product
	Movements []Movement
}

// NewInventory builds an empty inventory.
func NewInventory() *Inventory {
	return &Inventory{Products: make(map[string]Product)}
}

// AddProduct registers a product.
func (i *Inventory) AddProduct(p Product) error {
	if _, ok := i.Products[p.SKU]; ok {
		return fmt.Errorf("%w: %s", ErrDuplicateSKU, p.SKU)
	}
	i.Products[p.SKU] = p
	return nil
}

// Get returns the registered product or an error.
func (i *Inventory) Get(sku string) (Product, error) {
	p, ok := i.Products[sku]
	if !ok {
		return Product{}, fmt.Errorf("%w: %s", ErrUnknownSKU, sku)
	}
	return p, nil
}

// Receive adds stock.
func (i *Inventory) Receive(sku string, quantity int, note string) error {
	if _, err := i.Get(sku); err != nil {
		return err
	}
	if quantity <= 0 {
		return ErrNonPositiveQty
	}
	i.Movements = append(i.Movements, Movement{
		SKU: sku, Kind: MovementReceive, Quantity: quantity, At: time.Now(), Note: note,
	})
	return nil
}

// Sell removes stock, failing if insufficient.
func (i *Inventory) Sell(sku string, quantity int, note string) error {
	if _, err := i.Get(sku); err != nil {
		return err
	}
	if quantity <= 0 {
		return ErrNonPositiveQty
	}
	onHand, _ := i.Quantity(sku)
	if onHand < quantity {
		return fmt.Errorf("%w: %s on hand %d, requested %d", ErrInsufficientStock, sku, onHand, quantity)
	}
	i.Movements = append(i.Movements, Movement{
		SKU: sku, Kind: MovementSell, Quantity: quantity, At: time.Now(), Note: note,
	})
	return nil
}

// Quantity is the aggregate of all movements for a sku.
func (i *Inventory) Quantity(sku string) (int, error) {
	if _, err := i.Get(sku); err != nil {
		return 0, err
	}
	total := 0
	for _, m := range i.Movements {
		if m.SKU == sku {
			total += m.Signed()
		}
	}
	return total, nil
}

// History returns the movement list for a sku.
func (i *Inventory) History(sku string) ([]Movement, error) {
	if _, err := i.Get(sku); err != nil {
		return nil, err
	}
	var out []Movement
	for _, m := range i.Movements {
		if m.SKU == sku {
			out = append(out, m)
		}
	}
	return out, nil
}

// TotalValue sums price × quantity across all products.
func (i *Inventory) TotalValue() float64 {
	total := 0.0
	for _, p := range i.Products {
		q, _ := i.Quantity(p.SKU)
		total += p.Price * float64(q)
	}
	return total
}

// RestockNeeded returns products at or below their reorder point.
func (i *Inventory) RestockNeeded() []Product {
	var out []Product
	for _, p := range i.Products {
		if p.ReorderPoint <= 0 {
			continue
		}
		q, _ := i.Quantity(p.SKU)
		if q <= p.ReorderPoint {
			out = append(out, p)
		}
	}
	return out
}

// ByCategory filters products.
func (i *Inventory) ByCategory(category string) []Product {
	var out []Product
	for _, p := range i.Products {
		if p.Category == category {
			out = append(out, p)
		}
	}
	return out
}
