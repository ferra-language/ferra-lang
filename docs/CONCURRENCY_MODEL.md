# Ferra Concurrency Model v0.1

> **Status:** Initial Draft - Module 2.1 (Steps 2.1.1, 2.1.2, 2.1.3)

## 1. Introduction & Goals

This document specifies the design for Ferra's concurrency model. Ferra aims to provide a concurrency system that is safe, efficient, understandable, and, crucially, **deterministic** by default for its core actor model features.

**Core Goals:**

*   **Determinism**: For a given input, concurrent programs built with Ferra's core actor model should produce the same output and exhibit the same behavior across multiple runs. This greatly aids testing, debugging, and reasoning about concurrent systems.
*   **Safety**: Leverage Ferra's ownership and borrowing system to prevent data races and other common concurrency pitfalls by design.
*   **Simplicity & Understandability**: Offer a clear and high-level model for concurrency that is easier to reason about than raw threads and locks.
*   **Performance**: While determinism and safety are primary, the model should allow for efficient execution and scalability.

This initial specification (v0.1) focuses on the foundational deterministic actor model, `async`/`await` integration for non-blocking operations within actors, and basic inter-actor communication mechanisms.

## 2. Core Concepts Overview

*   **Actors**: Instances of an `actor` behavior definition. They are isolated, independently executing entities that own their state (an instance of a Ferra `data` type) and communicate exclusively through asynchronous message passing. Actors process messages one at a time from their mailbox.
*   **Actor Behaviors**: Defined using the `actor` keyword, these specify the state initialization logic (`fn init`) and message handling functions (`async fn handle_...`) for a class of actors.
*   **Actor State**: An instance of a Ferra `data` type, privately owned by an actor instance. Message handlers receive the current state and return a new state.
*   **Messages**: Immutable data structures (Ferra `data` types) or data with clear ownership transfer semantics passed between actors.
*   **Deterministic Scheduler**: A conceptual component responsible for orchestrating actor execution. For Ferra's deterministic model, the scheduler ensures repeatable behavior by controlling the order of message delivery and processing across actors, based on a fixed policy or compile-time analysis.
*   **`async`/`await`**: Keywords used within actor message handlers (`async fn handle_...`) to perform non-blocking operations. `await` yields control, allowing the actor system to process other work (e.g., other actors, other messages for the current actor if re-entrant calls are allowed and ordered by the scheduler) without blocking an OS thread.
*   **Channels**: Unidirectional or bidirectional conduits for sending messages between specific actors, providing a typed and explicit communication path. Often used for direct replies or specific stream-like interactions.
*   **`ActorRef<BehaviorType>`**: An opaque, shareable reference to a spawned actor instance. Used to send messages to the actor.

## 3. Actor Model Details (Step 2.1.1)

Ferra's actor model emphasizes a clear separation between an actor's behavior (its logic) and its state. This promotes a more functional approach to state management within actors and aids in achieving determinism.

### 3.1. Actor Behavior Definition

An actor's behavior is defined using the `actor` keyword, followed by a name for the behavior. It contains an `init` function to set up initial state and one or more `async fn handle_...` functions for processing messages.

```ferra
// Message Definitions (standard Ferra `data` types)
data Increment {}
data Add { amount: Int }
data GetCount {} // A query message that expects a CurrentValue reply
data CurrentValue { value: Int } // A reply message type

// Definition for the actor's state
data CounterState {
    count: Int,
}

// Actor Behavior Definition
actor CounterBehavior {
    // Initial state for a new actor instance of this behavior.
    // This function is called by the actor system when an actor is spawned.
    // It returns the initial state data.
    fn init(start_val: Int) -> CounterState {
        // Logging or side effects during init should be carefully considered
        // for determinism if they interact with external non-deterministic systems.
        // For now, assume println is a deterministic logging facility for this context.
        println("CounterBehavior: Initializing state with " + String::from_int(start_val));
        return CounterState { count: start_val };
    }

    // Message handlers are `async fn` to allow `await` for non-blocking operations.
    // They receive the current actor state (`self_state`) by value and the message.
    // They MUST return the new state of the actor (which can be the same as self_state).
    async fn handle_increment(self_state: CounterState, _message: Increment) -> CounterState {
        let new_count = self_state.count + 1;
        println("CounterBehavior: Count incremented to " + String::from_int(new_count));
        return CounterState { count: new_count }; // Return the new state
    }

    async fn handle_add(self_state: CounterState, message: Add) -> CounterState {
        let new_count = self_state.count + message.amount;
        println("CounterBehavior: Count is now " + String::from_int(new_count));
        return CounterState { count: new_count };
    }

    // For messages expecting a reply, the handler can send a reply message.
    // This handler also returns the (unchanged) state.
    // The `reply_channel_sender` is provided by the runtime for an 'ask' pattern.
    async fn handle_get_count(self_state: CounterState, _message: GetCount, replier: ActorRef<CurrentValue>) -> CounterState {
        println("CounterBehavior: Replying with current count " + String::from_int(self_state.count));
        // The `!` operator is the message send operator.
        replier ! CurrentValue { value: self_state.count }; 
        return self_state; // State is unchanged by this query
    }
}
```

**Key Characteristics:**
*   **State Isolation**: The `CounterState` is managed by the actor system for each instance of `CounterBehavior`. It is not directly shared.
*   **Functional State Updates**: Message handlers receive the current state and return the new state. This makes state transitions explicit.
*   **Message Matching**: The actor system (runtime) will be responsible for dispatching an incoming message to the appropriate `handle_...` function based on the message's type. (Exact mechanism TBD - e.g., convention over an internal registry).

### 3.2. Actor Spawning and `ActorRef`

An actor instance is created (spawned) by the actor system. Spawning an actor involves specifying its behavior and providing arguments for its `init` function.

*   **Spawning**: A keyword or runtime function like `spawn` is used.
    ```ferra
    // Conceptual spawning
    let counter_ref: ActorRef<CounterBehavior> = spawn CounterBehavior::init(start_val: 0);
    ```
*   **`ActorRef<BehaviorType>`**: This is an opaque, shareable handle to a specific actor instance. It is used to send messages to that actor. It is type-safe, ensuring only messages understood by `CounterBehavior` (i.e., those with corresponding `handle_...` methods) can be sent, though this check might be at the handler dispatch level rather than statically enforced on all `ActorRef` sends for arbitrary messages.

### 3.3. Message Sending

Messages are sent to actors using their `ActorRef`.

*   **Fire-and-Forget Send (Tell Pattern)**:
    *   Uses the `!` operator (inspired by Erlang/Akka, syntax TBD or a method call).
    *   `counter_ref ! Increment;`
    *   `counter_ref ! Add { amount: 5 };`
    *   The send is asynchronous. The sender does not wait for the message to be processed nor for a reply (unless designed into the message protocol itself, e.g. by including an `ActorRef` for a reply).

*   **Send and Receive Reply (Ask Pattern)**:
    *   For messages that logically require a reply (like `GetCount` which expects a `CurrentValue`).
    *   A higher-level `ask` operation, likely built on top of the basic send and a temporary reply mechanism (e.g., a private, temporary channel or future/promise).
    *   `let current_val_msg: CurrentValue = await counter_ref.ask(GetCount);` (Conceptual)
    *   The `ask` operation would involve sending the message (e.g., `GetCount`) along with information about where to send the reply. The actor system and the `handle_get_count` (with its `replier` parameter) would collaborate to deliver the `CurrentValue` message back.
    *   The `await` here signifies that the asking actor/task pauses until the reply is received.

### 3.4. Actor Lifecycle & Mailbox

*   **Mailbox**: Each actor instance has an implicit, private mailbox where incoming messages are enqueued.
*   **Message Processing**: The actor processes messages from its mailbox one at a time, in an order determined by the deterministic scheduler.
*   **State Management**: After a handler processes a message and returns a new state, the actor system updates the actor instance's internal state to this new value before processing the next message.
*   **Termination**: (TBD for v0.1) How actors are stopped (e.g., poison pill message, `ActorRef.stop()`) and any `terminate` handlers are invoked.
*   **Error Handling**: (TBD for v0.1) Behavior when a message handler panics or returns an unhandled error. Simple crash/log initially, with supervision as a future possibility.

*(Further details on actor supervision, error kernels, and advanced lifecycle events are out of scope for v0.1 but are future considerations.)*

## 4. `async`/`await` Syntax and Semantics (Step 2.1.2)

Ferra integrates `async`/`await` to enable non-blocking operations primarily within actor message handlers, ensuring that actors remain responsive and the system can efficiently manage concurrent tasks without relying on a large number of OS threads.

### 4.1. `async fn` Declaration

Functions (including actor message handlers like `handle_...`) that need to perform `await` operations must be declared as `async fn`.

```ferra
actor MyActorBehavior {
    // ... init ...
    async fn handle_my_message(self_state: MyState, msg: MyMessage) -> MyState {
        // ... can use await here ...
        let result = await some_async_operation();
        // ... process result ...
        return new_state;
    }
}
```
*   The `async` keyword on a function implies that it returns a future-like value implicitly, which is managed by the Ferra runtime and scheduler.
*   Within an `async fn`, `await` can be used to pause execution until an awaitable operation completes.

### 4.2. `await` Expression Syntax

Ferra uses a postfix `await` keyword for expressions:

*   **Syntax**: `Expression.await`
    *   This is consistent with the `AwaitExpr ::= Expression "." "await"` rule in `docs/SYNTAX_GRAMMAR_V0.1.md`.
*   **Error Propagation with `await`**: The `?` operator can be combined with `await` for concise error handling if the awaited expression yields a `Result<T,E>`:
    *   `let value = some_async_result_operation().await?;`
    *   This is equivalent to: `let temp_result = some_async_result_operation().await; let value = temp_result?;`

### 4.3. Semantics of `await` within an Actor

When an `await` expression is encountered within an `async fn` (typically an actor's message handler):

1.  **Suspension**: The execution of the current `async fn` task is suspended at the `await` point.
2.  **Yield Control**: Control is yielded back to the actor system's scheduler. The actor instance does **not** block an OS thread.
3.  **Scheduler Action**: The scheduler can then execute other ready actors or process other events.
4.  **Resumption**: When the operation being `await`ed completes (e.g., an async I/O operation finishes, a message reply is received via an internal future tied to an `ask` pattern), the scheduler is notified.
5.  The scheduler will eventually make the suspended task runnable again and resume its execution from the point immediately after the `await` with the result of the completed operation.
6.  **Message Processing Order**: For v0.1, an actor will fully process one message (potentially through multiple `await` suspension/resumption cycles for that single message's handler) before starting the next message from its mailbox. This simplifies the initial model for reasoning about state changes within an actor due to a single message. Re-entrant processing of new messages while a previous message handler for the same actor is suspended is a more advanced topic deferred beyond v0.1 to ensure deterministic behavior is well-understood first.

### 4.4. Awaitable Operations (v0.1)

Initially, the following types of operations are expected to be `await`able in Ferra v0.1:

*   **Actor `ask` operations**: `await actor_ref.ask(QueryMessage)` - waiting for a reply from another actor.
*   **Channel `receive` operations**: `await channel.receive()` - waiting for a message on a channel.
*   **Asynchronous I/O**: Operations from the standard library designed for non-blocking I/O (e.g., `await file.read_chunk_async()`). This is linked to TBD `STDLIB-IO-ASYNC-1` and `CONCURRENCY-IO-MODEL-1`.
*   **Calls to other `async fn`s**: `await another_async_function(args);`

The exact set of awaitable types will be defined by which types implement a specific runtime mechanism or trait (e.g., a `Future` trait, though this might be internal to the compiler/runtime for v0.1 to keep things simple).

### 4.5. Compiler Transformation

`async fn` declarations are typically transformed by the compiler into state machines (conceptually similar to generators or coroutines):

*   Each `await` point represents a potential yield point in the state machine.
*   Local variables that need to persist across `await` suspension points are captured as part of the state machine's state.
*   The runtime system manages the execution and scheduling of these state machines.

### 4.6. Interaction with Deterministic Scheduler

The deterministic scheduler plays a key role:

*   It tracks which actor tasks are runnable and which are suspended (waiting on an `await`).
*   When an awaited operation completes, the scheduler transitions the corresponding task to runnable.
*   Crucially for determinism, the order in which the scheduler picks runnable tasks (both newly ready tasks and those that were suspended) for execution **must be fixed and repeatable** given the same initial state and sequence of external events (if any).
*   This might involve a canonical ordering of actors, a fixed polling order, or a compile-time generated schedule for certain interactions.

*(Further details on the compiler transformation and specific scheduler algorithms are covered under TBDs `CONCURRENCY-AWAIT-IMPL-1` and `CONCURRENCY-SCHED-1` respectively.)*

## 5. Inter-Actor Communication (Channels - Step 2.1.3)

While actors primarily communicate by sending messages to `ActorRef` handles, channels provide a more direct, typed conduit for point-to-point or specific producer-consumer patterns between actors or asynchronous tasks. Channels are particularly useful for streaming data or when a reply mechanism more explicit than the `ask` pattern is desired.

### 5.1. Channel Concepts and Types

*   **`Channel<T>`**: A generic concept representing a communication channel that can transmit values of type `T`.
*   **Sender and Receiver Halves**: A channel typically consists of two distinct handles:
    *   `ChannelSender<T>`: Used to send messages into the channel.
    *   `ChannelReceiver<T>`: Used to receive messages from the channel.
    This separation allows the sending and receiving capabilities to be distributed to different actors or tasks.
*   **Message Type `T`**: The type `T` of the messages transferred over the channel must adhere to Ferra's ownership rules for data transfer (typically requiring `T` to be `Send`-able if actors could be on different threads, though for v0.1 deterministic single-scheduler model, this might be relaxed to simple ownership transfer/move semantics).

### 5.2. Channel Creation

Channels are created via a factory function, which returns the sender and receiver pair.

```ferra
// Example: Creating a channel for String messages with a bounded capacity
let (tx, rx): (ChannelSender<String>, ChannelReceiver<String>) = Channel::new(capacity: 16);
```
*   **`Channel::new<T>(capacity: Int) -> (ChannelSender<T>, ChannelReceiver<T>)`**: 
    *   Creates a new **bounded, MPSC (Multiple Producer, Single Consumer)** channel with the specified `capacity` for messages of type `T`.
    *   For v0.1, all channels are bounded to encourage backpressure-aware designs. A capacity of 0 could mean a rendezvous channel if supported, but a minimum capacity of 1 is typical for bounded buffers.
    *   The `ChannelSender<T>` can typically be cloned to allow multiple producers.
    *   The `ChannelReceiver<T>` is unique, ensuring only one consumer.

### 5.3. Sending Messages

Messages are sent via the `ChannelSender<T>`.

```ferra
// Assuming tx is a ChannelSender<String>
let send_result = await tx.send("Hello, channel!");
match send_result {
    Ok(()) => { /* message sent successfully or enqueued */ }
    Err(SendError::Closed) => { eprintln("Channel was closed, message not sent."); }
}
```
*   **`async sender.send(message: T) -> Result<(), SendError>`**: 
    *   Sends `message` into the channel. Ownership of `message` is moved.
    *   This operation is `async` because if the channel's buffer is full (for bounded channels), the `send` call will suspend until space becomes available or the channel is closed.
    *   Returns `Ok(())` if the message was successfully enqueued.
    *   Returns `Err(SendError::Closed)` if the channel (specifically, its receiver half) has been closed and can no longer accept messages.
*   `data SendError { Closed }` (Conceptual error type)

### 5.4. Receiving Messages

Messages are received via the `ChannelReceiver<T>`.

```ferra
// Assuming rx is a ChannelReceiver<String>
loop {
    match await rx.receive() {
        Ok(message) => {
            println("Received: " + message);
        }
        Err(ReceiveError::ClosedAndEmpty) => {
            println("Channel closed and empty, no more messages.");
            break;
        }
    }
}
```
*   **`async receiver.receive() -> Result<T, ReceiveError>`**: 
    *   Suspends execution (`await`s) until a message is available in the channel or the channel is closed and empty.
    *   Returns `Ok(message: T)` with the received message. Ownership of the message is moved to the caller.
    *   Returns `Err(ReceiveError::ClosedAndEmpty)` if the channel is closed (all senders have been dropped or `close()` has been called on a sender/receiver) AND the buffer is empty.
*   `data ReceiveError { ClosedAndEmpty }` (Conceptual error type)

### 5.5. Ownership and Data Transfer

*   When a value of type `T` is sent via `sender.send(message: T)`, ownership of `message` is **moved** into the channel.
*   When `receiver.receive()` successfully returns `Ok(message: T)`, ownership of `message` is **moved** out of the channel to the receiver.
*   This ensures that there is always a single owner for the data, aligning with Ferra's core memory safety principles and preventing data races if channels were to cross thread boundaries in the future (though v0.1 focuses on a single-scheduler model).

### 5.6. Closing Channels

*   **Implicit Close**: A channel is typically considered closed for receiving when all `ChannelSender<T>` instances associated with it have been dropped.
*   **Explicit Close (TBD)**: An explicit `sender.close()` or `receiver.close()` method might be provided. Closing the receiver or all senders would signal to the other end(s) that no more messages will be sent/can be received.
    *   `sender.close()`: Informs the receiver(s) that no more messages will be sent. Subsequent `receive` calls will drain the buffer and then return `Err(ReceiveError::ClosedAndEmpty)`.
    *   `receiver.close()`: Informs sender(s) that no more messages will be received. Subsequent `send` calls would return `Err(SendError::Closed)`.
*   For v0.1, relying on dropping all senders to close the channel for receivers is a sufficient minimal mechanism.

*(Further details on different channel types like SPSC, MPMC, or unbounded channels, and more advanced features like `try_send`/`try_receive` are deferred beyond v0.1.)*

## 6. Determinism: Principles and Enforcement

Determinism is a core differentiating feature of Ferra's concurrency model. It ensures that for the same inputs, a concurrent program will always produce the same outputs and exhibit the same behavior, making concurrent systems more testable, debuggable, and reliable.

### 6.1. Compile-Time Scheduling Analysis

*   **Static Analysis**: The Ferra compiler performs static analysis on actor networks to determine potential message flows and dependencies between actors.
*   **Message Flow Graph**: Constructs a directed graph representing potential message passing between actors, identifying potential race conditions or non-deterministic behaviors.
*   **Schedule Generation**: For closed actor systems (those without external inputs), the compiler can generate a fixed, deterministic execution schedule at compile-time.

### 6.2. Runtime Determinism Enforcement

*   **Fixed Scheduling Policy**: The Ferra runtime employs a deterministic scheduling algorithm that ensures messages are processed in a consistent order across runs.
*   **Logical Clock**: Uses a deterministic logical clock or timestamp mechanism to order events in a repeatable sequence.
*   **Message Queue Ordering**: Messages are enqueued into actor mailboxes according to a well-defined ordering policy (e.g., based on sender ID and a monotonically increasing message counter).

### 6.3. External Input Handling

*   **Input Serialization**: External inputs (e.g., from I/O) are serialized into the actor system through designated input actor gateways.
*   **Input Tagging**: Each external input is tagged with metadata (e.g., arrival timestamp, source) that is used by the scheduler to ensure consistent processing order.
*   **Replay Capability**: The system can record external inputs to enable deterministic replay for debugging.

### 6.4. Explicit Non-Determinism Opt-In

*   **Explicit Annotation**: Developers can explicitly mark parts of the system as potentially non-deterministic with `#[nondet]` attributes.
*   **Isolation**: Non-deterministic components are isolated and their effects on the deterministic parts of the system are carefully controlled.
*   **Documentation Requirement**: Code that opts out of determinism must document the reason and potential impacts.

## 7. Memory Model & Ownership Considerations

The interaction between Ferra's concurrency model and its ownership system is crucial for preventing data races while enabling efficient message passing.

### 7.1. Ownership Rules for Messages

*   **Move Semantics**: By default, when a message is sent to an actor or through a channel, ownership is moved from the sender to the receiver.
*   **Clone for Shared Data**: For cases where the same data needs to be sent to multiple actors, explicit cloning is required.
*   **No Implicit Sharing**: Ferra does not allow implicit sharing of mutable data between actors; all communication must be explicit through messages or channels.

### 7.2. Message Type Requirements

*   **Compile-Time Verification**: Message types are verified at compile time to ensure they can be safely transferred between actors.
*   **`Send` Trait**: In the future, when multi-threaded execution is supported, message types will need to implement a `Send` trait (similar to Rust) to ensure thread safety.
*   **Immutability Preference**: While not strictly required, immutable data structures (Ferra `data` types) are preferred for messages for simplicity and safety.

### 7.3. Borrowing and Lifetimes with `async`/`await`

*   **Extended Lifetimes**: The compiler automatically extends lifetimes of borrowed data across `await` points as needed.
*   **Borrowing Restrictions**: Certain borrowing patterns that could lead to problems across `await` points are disallowed (similar to Rust's borrowing rules with async).
*   **Static Verification**: The borrow checker verifies at compile time that all borrows are valid across the entire asynchronous execution flow.

### 7.4. Memory Safety Guarantees

*   **Data Race Freedom**: The ownership model ensures that there can never be data races, as mutable data is never shared between actors without explicit transfer of ownership.
*   **Deadlock Prevention**: The actor model with message passing inherently prevents traditional deadlocks from lock acquisition order issues.
*   **Memory Leak Prevention**: The ownership system tracks resources to ensure they are properly released when no longer needed.

## 8. Standard Library Support

Ferra's standard library provides comprehensive support for the concurrency model through well-defined APIs and utilities.

### 8.1. Actor System APIs

*   **Actor Management**:
    ```ferra
    // Core actor system functions
    namespace actors {
        // Spawn an actor with the given behavior and init arguments
        fn spawn<B: ActorBehavior>(init_args...) -> ActorRef<B>
        
        // Gracefully stop an actor
        async fn stop<B: ActorBehavior>(actor: ActorRef<B>)
    }
    ```

*   **Actor System Configuration**:
    ```ferra
    // Configuration options for the actor system
    data ActorSystemConfig {
        // Maximum number of logical threads for the actor system
        thread_pool_size: Int = 4,
        // Buffer size for actor mailboxes
        default_mailbox_capacity: Int = 32,
        // Strategy for handling mailbox overflow
        mailbox_overflow_strategy: OverflowStrategy = OverflowStrategy::Backpressure,
    }
    
    // Initialize the actor system with the given configuration
    fn init_actor_system(config: ActorSystemConfig) -> ActorSystem
    ```

### 8.2. Channel API Extensions

*   **Additional Channel Types**:
    ```ferra
    // Create a Single-Producer Single-Consumer channel
    fn Channel::new_spsc<T>(capacity: Int) -> (ChannelSender<T>, ChannelReceiver<T>)
    
    // Create a broadcast channel (one-to-many)
    fn Channel::new_broadcast<T>(capacity: Int) -> (BroadcastSender<T>, fn() -> BroadcastReceiver<T>)
    ```

*   **Non-Blocking Channel Operations**:
    ```ferra
    // Try to send without awaiting
    fn ChannelSender<T>::try_send(message: T) -> Result<(), TrySendError>
    
    // Try to receive without awaiting
    fn ChannelReceiver<T>::try_receive() -> Result<T, TryReceiveError>
    ```

### 8.3. Asynchronous I/O Integration

*   **File I/O**:
    ```ferra
    // Asynchronous file operations
    namespace io::fs {
        fn open_async(path: String) -> Result<AsyncFile, IOError>
        
        trait AsyncFile {
            async fn read(&mut self, buffer: &mut [Byte]) -> Result<Int, IOError>
            async fn write(&mut self, buffer: &[Byte]) -> Result<Int, IOError>
            async fn flush(&mut self) -> Result<(), IOError>
        }
    }
    ```

*   **Network I/O**:
    ```ferra
    // Asynchronous networking
    namespace io::net {
        async fn connect(address: String, port: Int) -> Result<AsyncTcpStream, NetworkError>
        
        trait AsyncTcpStream {
            async fn read(&mut self, buffer: &mut [Byte]) -> Result<Int, NetworkError>
            async fn write(&mut self, buffer: &[Byte]) -> Result<Int, NetworkError>
        }
    }
    ```

### 8.4. Time and Timers

*   **Deterministic Time API**:
    ```ferra
    namespace time {
        // Get the current logical time
        fn now() -> LogicalTime
        
        // Create a timer that completes after the specified duration
        async fn delay(duration: Duration) -> ()
        
        // Schedule a recurring action
        fn schedule_recurring(interval: Duration, action: fn() -> ()) -> TimerId
    }
    ```

## 9. Examples

The following examples demonstrate how to use Ferra's concurrency model in practice.

### 9.1. Basic Actor Example

A simple ping-pong example demonstrating actors exchanging messages:

```ferra
// Message definitions
data Ping { count: Int }
data Pong { count: Int }

// Ping actor behavior
data PingState {
    pongs_received: Int,
}

actor PingActor {
    fn init() -> PingState {
        return PingState { pongs_received: 0 };
    }
    
    async fn handle_start(self_state: PingState, pong_actor: ActorRef<PongActor>) -> PingState {
        println("PingActor: Starting ping-pong sequence");
        pong_actor ! Ping { count: 0 };
        return self_state;
    }
    
    async fn handle_pong(self_state: PingState, msg: Pong, pong_actor: ActorRef<PongActor>) -> PingState {
        let new_count = msg.count + 1;
        let new_pongs = self_state.pongs_received + 1;
        
        println("PingActor: Received Pong #" + String::from_int(msg.count));
        
        if new_pongs < 10 {
            println("PingActor: Sending Ping #" + String::from_int(new_count));
            pong_actor ! Ping { count: new_count };
        } else {
            println("PingActor: Ping-pong sequence complete");
        }
        
        return PingState { pongs_received: new_pongs };
    }
}

// Pong actor behavior
data PongState {
    pings_received: Int,
}

actor PongActor {
    fn init() -> PongState {
        return PongState { pings_received: 0 };
    }
    
    async fn handle_ping(self_state: PongState, msg: Ping, ping_actor: ActorRef<PingActor>) -> PongState {
        let new_pings = self_state.pings_received + 1;
        
        println("PongActor: Received Ping #" + String::from_int(msg.count));
        println("PongActor: Sending Pong #" + String::from_int(msg.count));
        
        ping_actor ! Pong { count: msg.count };
        
        return PongState { pings_received: new_pings };
    }
}

// Main function to set up the actor system
fn main() {
    // Initialize the actor system
    let system = init_actor_system(ActorSystemConfig {});
    
    // Spawn our actors
    let pong = spawn PongActor::init();
    let ping = spawn PingActor::init();
    
    // Start the ping-pong sequence
    ping ! StartPingPong { pong_actor: pong };
    
    // Wait for the system to complete
    system.await_termination();
}
```

### 9.2. Channel Communication Example

An example demonstrating channel-based communication between tasks:

```ferra
// A producer-consumer pattern using channels
async fn producer(tx: ChannelSender<Int>) -> Result<(), SendError> {
    for i in 0..10 {
        println("Producer: Producing value " + String::from_int(i));
        await time::delay(Duration::from_millis(100));
        await tx.send(i)?;
    }
    println("Producer: Done producing values");
    return Ok(());
}

async fn consumer(rx: ChannelReceiver<Int>) -> Result<(), ReceiveError> {
    loop {
        match await rx.receive() {
            Ok(value) => {
                println("Consumer: Consumed value " + String::from_int(value));
            },
            Err(ReceiveError::ClosedAndEmpty) => {
                println("Consumer: Channel closed, stopping");
                break;
            }
        }
    }
    return Ok(());
}

fn main() {
    // Create a channel
    let (tx, rx) = Channel::new::<Int>(capacity: 5);
    
    // Spawn our producer and consumer tasks
    spawn_task(producer(tx));
    spawn_task(consumer(rx));
    
    // Wait for all tasks to complete
    await_all_tasks();
}
```

### 9.3. Combining Actors with async/await

An example showing how to combine actors with asynchronous I/O:

```ferra
data FileRequest { path: String, replier: ActorRef<FileResponse> }
data FileResponse { content: Result<String, IOError> }

data FileReaderState {}

actor FileReaderActor {
    fn init() -> FileReaderState {
        return FileReaderState {};
    }
    
    async fn handle_file_request(self_state: FileReaderState, msg: FileRequest) -> FileReaderState {
        println("FileReaderActor: Reading file " + msg.path);
        
        // Perform asynchronous file I/O
        let content_result = match await io::fs::open_async(msg.path) {
            Ok(mut file) => {
                let mut buffer = Vector::<Byte>::new();
                match await file.read_to_end(&mut buffer) {
                    Ok(_) => Ok(String::from_bytes(buffer)),
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        };
        
        // Send the response back
        msg.replier ! FileResponse { content: content_result };
        
        return self_state;
    }
}

data ClientState {}

actor ClientActor {
    fn init() -> ClientState {
        return ClientState {};
    }
    
    async fn handle_start(self_state: ClientState, file_reader: ActorRef<FileReaderActor>) -> ClientState {
        println("ClientActor: Requesting file content");
        
        // Create a temporary actor for the reply
        let reply = await file_reader.ask(FileRequest { 
            path: "example.txt"
        });
        
        // Process the response
        match reply.content {
            Ok(content) => println("ClientActor: Received file content: " + content),
            Err(e) => println("ClientActor: Error reading file: " + e.to_string())
        }
        
        return self_state;
    }
}
```

## 10. Out of Scope for Initial Version (v0.1 Concurrency)

*   Distributed actors (across machine boundaries).
*   Complex, dynamic supervision trees and fault tolerance strategies beyond simple actor error handling.
*   Shared-memory concurrency primitives (e.g., threads, mutexes, atomics directly exposed to users – the actor model is the primary high-level abstraction).
*   Non-deterministic execution modes (unless explicitly opted into via a separate mechanism).

## 11. Open Questions / TBD (Concurrency Model v0.1)

| Tag                       | Issue                                                                                                                                                                       |
|---------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| CONCURRENCY-ERR-PROP-1    | Detailed error propagation mechanisms for `async` operations and inter-actor messages (e.g., how errors from an awaited future or a failed actor message send are handled). |
| CONCURRENCY-SCHED-IMPL-1  | Specific algorithms and data structures for the compile-time scheduler.                                                                                                     |
| CONCURRENCY-ACTOR-ID-1    | Representation and management of Actor IDs.                                                                                                                                 |
| CONCURRENCY-DISTRIB-1     | Initial mechanisms for actor placement hints if considering distributed scenarios beyond v0.1.                                                                              |
| CONCURRENCY-IO-URING-1    | Feasibility and design for integrating Linux `io_uring` for high-performance asynchronous I/O within the deterministic actor model.                                       |
| CONCURRENCY-SEC-MODEL-1   | Interaction with Ferra's security model, particularly if actors could have distinct permission sets or operate in different sandboxes (see `SECURITY_MODEL.md` §5.1 and TBD SEC-ACTOR-PERM-1). This is a future consideration for finer-grained security. |

---
This document outlines the design for Ferra's deterministic concurrency model based on compile-time scheduled actors and `async`/`await` syntax. 