## Purpose

Defines the payload with which a producer asks Edge to erase messages it published from the events stream.

## ADDED Requirements

### Requirement: The sequence list is bounded, positive and strictly ascending

`PlatformStreamErasureRequested::validate` SHALL require 1 to 500 sequences, each at least 1 and at most 2^53 - 1, strictly ascending.

#### Scenario: An empty list is rejected
- **WHEN** `stream_sequences` is empty
- **THEN** `validate()` returns `EmptyStreamSequences`

#### Scenario: A list of 501 is rejected and 500 is accepted
- **WHEN** `stream_sequences` holds 501 ascending values
- **THEN** `validate()` returns `TooManyStreamSequences`, and exactly 500 is accepted

#### Scenario: A zero, a descending list, a duplicate or an unsafe integer is rejected
- **WHEN** an entry is 0, the list is not ascending, an entry repeats or an entry exceeds 2^53 - 1
- **THEN** `validate()` returns the matching error

### Requirement: The request names a privacy request

`PlatformStreamErasureRequested::validate` SHALL require `request_ref` to be of kind `privacy_request`.

#### Scenario: A request reference of another kind is rejected
- **WHEN** `request_ref` is not of kind `privacy_request`
- **THEN** `validate()` returns `StreamErasureRequestKind`
