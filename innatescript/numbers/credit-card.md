# Credit Card Validator

Structural verification. The Luhn algorithm doesn't prove a card is real — it proves the number is well-formed. Catches typos and transpositions, not fraud. This is the first project about trust boundaries.

## Resolver native

```dpn
@luhn_check{number: @card_number} -> @valid
@identify_network{number: @card_number} -> @network
@validate_card{number: @card_number} -> @result
```

## Where gates as structural verification

Credit card validation is the clearest analog to InnateScript's `where` gates. A `where` can verify that an agent's output is structurally valid — correct format, passes checksums, meets schema — without verifying that it's TRUE. Structural validity vs. semantic validity.

The Luhn check is a `<-` gate: a cheap local test that rejects obviously invalid inputs before they reach an expensive remote operation (hitting the payment processor). InnateScript's verification gates work the same way — `<-` gates filter before costly operations, `where` scores after.

```dpn
[payment_intake @transaction
    <- @validate_card{number: @transaction:card_number}
       -> @validation
       where [valid == true]
    @identify_network{number: @transaction:card_number}
       -> @network
    -> @network:handler
] where [card passes structural verification before network routing]
```

The `<-` gate on `validate_card` is the Luhn check — it rejects malformed numbers before anything else runs. The `where` on the outer choreography is the structural guarantee: no card reaches network routing unless it's well-formed.

## Network identification as dispatch

Identifying Visa/Mastercard/Amex/Discover is pattern matching on prefixes. This is dispatch — routing to the correct handler based on input shape. InnateScript's resolver does the same thing when it looks at `@reference{...}` and decides which native or agent handles it. The card network IS the dispatcher.

```dpn
[process_payment @order
    <- @validate_card{number: @order:card} -> @validation
       where [valid == true]
    match @validation:network [
        "Visa"       -> @VisaProcessor{charge @order:amount}
        "Mastercard" -> @MCProcessor{charge @order:amount}
        "Amex"       -> @AmexProcessor{charge @order:amount, verify premium status}
        "Discover"   -> @DiscoverProcessor{charge @order:amount}
        _            -> reject "unsupported network"
    ] -> @charge_result
    <- @ElianaRiviera{verify charge confirmation is genuine}
] where [payment processed through correct network handler]
```

The `match` on network is dispatch. Each processor is a different agent with different capabilities — Amex verifies premium status, others don't. The network prefix determines the handler, just like InnateScript's resolver dispatches based on reference shape.

## Information hiding: masking as access control

Masking (`****1234`) is the first encounter with information hiding in the Rosetta Stone. Not all agents need the full card number. The choreography should pass masked values to agents who don't need the real data. This is access control at the data level.

```dpn
[payment_with_audit @order
    <- @validate_card{number: @order:card} -> @validation
    @ChargeAgent{
        full_number: @order:card,
        amount: @order:amount
    } -> @charge_result
    concurrent [
        @AuditLogger{card: @validation:display, amount: @order:amount}
        @CustomerNotifier{card: @validation:display, status: @charge_result:status}
    ]
] where [only ChargeAgent sees full card number; audit and notification use masked display]
```

The `where` here isn't about computation — it's about information flow. `ChargeAgent` gets the real number because it needs it. `AuditLogger` and `CustomerNotifier` get the masked display because they don't. The choreography enforces who sees what.

## Progressive trust: the payment choreography

The full payment flow shows how `<-` gates implement progressive trust verification. Each gate is cheaper than the next, and each filters before the expensive operation that follows.

```dpn
[full_payment_flow @order
    <- @validate_card{number: @order:card} -> @validation
       where [valid == true]                              ;; gate 1: structural (free, local)
    <- @FraudDetector{
        card: @validation:display,
        amount: @order:amount,
        ip: @order:ip
    } -> @fraud_score
       where [score < 0.7]                                ;; gate 2: risk (cheap, ml model)
    <- @NetworkProcessor{
        card: @order:card,
        amount: @order:amount,
        network: @validation:network
    } -> @auth_result
       where [authorized == true]                         ;; gate 3: authorization (expensive, remote)
    @SettlementQueue{enqueue @auth_result}
] where [each gate is cheaper than the next; structural before risk before authorization]
```

Three gates, ascending cost. The Luhn check is free. Fraud detection is a local model call. Authorization hits the payment network. Each `<-` gate prevents the next expensive operation from running on bad input. This is the same pattern as InnateScript's verification pipeline: cheap filters first, expensive verification last.

## Design note

Thirteen projects in. This is the first one about trust and access control rather than pure computation. The Luhn algorithm is trivial — the insight is that validation is gating, gating is filtering, and filtering is what `<-` gates do. The card validator isn't a math problem. It's the first stage of a trust pipeline, and InnateScript's choreographic structure maps directly onto how payment systems actually work: verify structure, assess risk, authorize, settle. Each stage is a gate. Each gate has a `where`.
