# Ferra Borrow Checker & Ownership UX Design v0.1

> **Status:** Initial Draft - Module 2.2 (Steps 2.2.1, 2.2.2, 2.2.3)
> **Last Reviewed:** 2024-07-29
> **Version:** 0.1

## 1. Introduction and Goals

Ferra's memory safety is built upon a compile-time ownership and borrowing system, designed to prevent common memory errors like dangling pointers, data races, and use-after-free without a garbage collector. The foundational concepts of this system are outlined in `OWNERSHIP_MODEL_PRINCIPLES_V0.1.md`, which emphasizes:

*   **Single Ownership**: Each value has one unique owner.
*   **Scope-Bound Lifetimes**: Values are dropped when their owner goes out of scope.
*   **Move Semantics**: Ownership is transferred by default for resource-managing types.
*   **Copy Semantics**: Simple data types (`Int`, `Float`, `Bool`, `Unit`, and structures composed of them) are copied.
*   **Borrowing**: Data can be accessed via shared (`&`) or exclusive (`&mut`) references without transferring ownership.

This document, `OWNERSHIP_BORROW_CHECKER.md`, refines these principles into a more detailed specification for Ferra's borrow checker. The primary goals of this specification are to:

1.  **Define Detailed Borrow Checking Rules**: Provide precise rules for how shared and mutable borrows are created, used, and validated by the compiler, including their interaction with lifetimes.
2.  **Ensure Seamless Integration with Concurrency**: Clarify how ownership and borrowing interact with Ferra's deterministic actor model and `async`/`await` features, as detailed in `CONCURRENCY_MODEL.md`, to maintain safety in concurrent contexts.
3.  **Deliver an Exceptional User Experience (UX)**: Design the borrow checker's diagnostic messages to be "positive-first" (as advocated in `Steps.md`, Section 3), actionable, and clear, guiding users toward correct code.
4.  **Specify AI Tag Integration**: Detail how AI-provided hints, such as `ai.assume(nll="noalias")` (from `Steps.md`, Section 2), can be used to inform the borrow checker in advanced or `unsafe` scenarios.

This specification aims to create a borrow checking system that is both robust in its safety guarantees and as intuitive as possible for developers familiar with modern systems programming concepts.

## 2. Core Borrowing Rules (Refined)

At the heart of Ferra's compile-time safety are the rules governing how data can be borrowed. These rules prevent data races and dangling references.

### 2.1. Fundamental Borrowing Principles

1.  **One Writer OR Multiple Readers**: For any given piece of data, at any point in time, the program can have:
    *   Exactly one exclusive (mutable) reference (`&mut T`), OR
    *   Any number of shared (immutable) references (`&T`).
    *   It is a compile-time error for these to coexist if their lifetimes overlap for the same piece of data (e.g., an exclusive reference cannot exist while any shared references are live, and vice-versa).
2.  **Borrows Cannot Outlive the Data (Dangling Reference Prevention)**: All references must point to valid, live data. The borrow checker ensures that no reference outlives the scope (lifetime) of the data it refers to.
    ```ferra
    // Conceptual example of a dangling reference error
    let r: &Int;
    {
        let x = 10; // x is owned within this scope
        r = &x;     // r borrows x
    } // x is dropped here. r now refers to deallocated memory.
    // println(*r); // ERROR: `r` would be a dangling reference if used here.
    ```

### 2.2. Shared (Immutable) Borrows (`&T`)

*   **Creation**: A shared borrow `&owner` can be created from any accessible owned value `owner` (whether bound by `let` or `var`) or from another existing shared borrow.
*   **Permissions**: Allows reading the data. Does not allow mutation of the data *through this reference*.
*   **Coexistence**: Multiple shared borrows to the same data can exist simultaneously.
*   **Interaction with Exclusive Borrows**: A shared borrow cannot be active if an exclusive (mutable) borrow to the same data is active or is created while the shared borrow is live and its scope has not ended.

    ```ferra
    let data_owner = String::from("hello"); // `data_owner` owns the String
    let r1 = &data_owner;       // OK: r1 is a shared borrow
    let r2 = &data_owner;       // OK: r2 is another shared borrow, coexists with r1
    println(*r1);               // OK: Can read through r1
    // *r1 = String::from("world"); // ERROR: Cannot mutate data through a shared borrow `&String`

    var mutable_owner = String::from("initial");
    let r3 = &mutable_owner;    // OK: Shared borrow from a `var` owner
    // mutable_owner = String::from("new"); // ERROR: Cannot reassign `mutable_owner` while `r3` (borrowing it) is live.
    println(*r3);
    // After `r3` is no longer live (e.g., end of scope or NLL), `mutable_owner` can be mutated or reassigned.
    ```

### 2.3. Exclusive (Mutable) Borrows (`&mut T`)

*   **Creation**: An exclusive borrow `&mut owner` can be created if and only if the *path* to `owner` is mutable:
    *   The `owner` is a `var` binding (e.g., `var x = data; let r_mut = &mut x;`).
    *   The `owner` is a field accessed through an existing exclusive borrow (e.g., `param_mut.field` where `param_mut` is `&mut ContainingStruct`).
    *   The `owner` is `self` in a method declared as `fn method(&mut self) { ... }`.
    *   A direct `&mut data_owner` from a `let data_owner = ...;` binding is **not** allowed, as `let` creates an immutable path to the owned data.
*   **Permissions**: Allows reading and mutating the data *through this reference*.
*   **Exclusivity**: If an exclusive borrow to data exists, no other borrows (shared or exclusive) to that same data can exist or be created while the exclusive borrow is live. This is enforced by the compiler to prevent data races.

    ```ferra
    var data = String::from("hello");
    let r_mut = &mut data; // OK: `data` is a `var`, providing a mutable path
    // let r_shared = &data; // ERROR: Cannot create a shared borrow while `r_mut` is active.
    // let r_mut2 = &mut data; // ERROR: Cannot create another exclusive borrow while `r_mut` is active.
    *r_mut = String::from("world"); // OK: Can mutate through `r_mut`
    println(*r_mut);
    // `r_mut`'s lifetime ends (e.g., end of scope or NLL analysis)
    let r_shared_after = &data; // OK now, as no exclusive borrow is active.
    println(*r_shared_after);
    ```
    ```ferra
    let immutable_path_owner = String::from("static");
    // let r_mut_direct = &mut immutable_path_owner; // ERROR: `immutable_path_owner` is a `let` binding (immutable path).
    ```

### 2.4. Lifetimes

Lifetimes define the scope or region of code for which a reference is valid. The borrow checker's primary role is to validate these lifetimes to prevent dangling references.

*   **Lexical Lifetimes (Scope-Based)**: In many cases, the lifetime of a reference is intuitively tied to the lexical scope where it is defined. A reference cannot be used outside the scope where its referent (the data it points to) is valid and alive.
    ```ferra
    let r_outer: &Int; // Declared in an outer scope
    {
        let x_inner = 10;    // x_inner is owned by this inner scope
        r_outer = &x_inner;  // r_outer borrows x_inner
        println(*r_outer);   // OK: r_outer and x_inner are valid here
    } // x_inner goes out of scope and is dropped. r_outer now refers to invalid memory.
    // println(*r_outer);   // ERROR: Use of `r_outer` here would be a use-after-free (dangling reference).
    ```
*   **Lifetime Inference**: Ferra's compiler will infer lifetimes in most common scenarios, reducing the need for explicit lifetime annotations by the programmer. The goal is to make common patterns "just work" without undue annotation burden.
*   **Non-Lexical Lifetimes (NLL) Principle**: While full NLL specification is complex (see Open Question TAG-BORROW-1), the underlying principle Ferra aims for is that a borrow is considered live only for the part of its lexical scope where it is actually used or could be used. This allows for more flexible code by ending borrows as soon as they are no longer needed, even if the reference variable itself is still lexically in scope.
    ```ferra
    var data = 10;
    let r1 = &mut data; // Exclusive borrow `r1` starts.
    *r1 = 20;           // `r1` is used here.
                        // After this point, `r1` is no longer used. NLL allows its borrow to effectively end.
    let r2 = &mut data; // OK: If NLL determines `r1`'s borrow has ended, a new exclusive borrow `r2` can start.
    *r2 = 30;
    println(data);      // Expected: 30
    ```
*   **Explicit Lifetime Annotations (Future)**: For complex scenarios, especially involving references in `data` structures and function signatures that return references or take multiple reference arguments, explicit lifetime annotations (e.g., `'a`, `'b`) will be necessary. These are deferred for a later, more advanced specification (as noted in `OWNERSHIP_MODEL_PRINCIPLES_V0.1.md` and TBD OB-1). Examples include:
    ```ferra
    // Conceptual future syntax with explicit lifetimes
    // fn get_ref<'a>(input: &'a String) -> &'a str; 
    // data MyStruct<'a> { field: &'a Something }
    ```

### 2.5. Borrowing Fields of `data` Structures

*   **Whole Struct Borrow (Default Conservative Approach)**: When a field of a `data` structure instance is borrowed, the borrow checker may conservatively consider the entire structure instance to be borrowed for the duration of the field borrow. This is especially true if the borrow is mutable, or if the compiler cannot easily prove disjointness.
    ```ferra
    data Point { x: Int, y: Int }
    var p = Point { x: 1, y: 2 };
    let y_ref_mut = &mut p.y; // This mutably borrows `p` as a whole to grant mutable access to `p.y`.
    // let x_ref_immut = &p.x; // ERROR: Cannot immutably borrow `p.x` because `p` is already mutably borrowed by `y_ref_mut`.
    *y_ref_mut = 3;
    // y_ref_mut's lifetime ends.
    let x_ref_after = &p.x; // OK now.
    println(*x_ref_after);
    ```
*   **Partial Borrows (Disjoint Fields - Aspirational Goal)**: Ferra aims to support partial borrows where feasible, allowing multiple borrows to different, provably disjoint fields of the same struct, even if one of those borrows is mutable. This significantly improves ergonomics.
    ```ferra
    data Point { x: Int, y: Int }
    var p = Point { x: 1, y: 2 };
    let x_ref_mut = &mut p.x; // Mutably borrows only p.x (assuming compiler can prove disjointness).
    let y_ref_immut = &p.y;   // OK: Immutably borrows p.y, which is disjoint from p.x.
    *x_ref_mut = 10;
    println(*y_ref_immut);    // Accesses y through its immutable borrow.
    ```
    The compiler must be able to statically prove that such field borrows are indeed disjoint for this to be safe. This may involve analysis of the structure's layout and the access paths.

### 2.6. Interaction with `let` (Immutable) and `var` (Mutable) Bindings

The mutability of a *binding* (path) is distinct from the mutability of a *reference*. `OWNERSHIP_MODEL_PRINCIPLES_V0.1.md` (Section 5) highlights that mutability is primarily a property of the binding (`var`).

*   **`let` Bindings (Immutable Path to Owned Data)**:
    *   If `let x = owned_value;`, then `x` establishes an immutable path to `owned_value`.
    *   Any number of shared borrows (`&x`) can be taken.
    *   An exclusive borrow (`&mut x`) **cannot** be directly taken from `x` if `x` is the direct owner of the data. The data is considered immutable *through this specific binding `x`*.
        ```ferra
        let s_owned = String::from("immutable path");
        let r1 = &s_owned; // OK
        // let r_mut = &mut s_owned; // ERROR: `s_owned` is a `let` binding, path is immutable.
        ```
    *   _Caveat_: If `x` is a `let` binding to an *existing reference* that is already mutable, then `x` is just an immutable alias to that mutable reference, and operations through `x` follow the original reference's mutability.
        ```ferra
        var original_mutable = String::from("original");
        let ref_to_mutable: &mut String = &mut original_mutable; // `ref_to_mutable` is an &mut T
        let x_alias = ref_to_mutable; // `x_alias` is an immutable binding to `ref_to_mutable`.
                                      // `x_alias` itself cannot be reassigned, but it holds an `&mut String`.
        *x_alias = String::from("changed via alias"); // OK: Mutating through the `&mut` reference held by `x_alias`.
        // x_alias = &mut another_string; // ERROR: `x_alias` binding itself is immutable.
        ```

*   **`var` Bindings (Mutable Path to Owned Data)**:
    *   If `var x = owned_value;`, then `x` establishes a mutable path to `owned_value`.
    *   Shared borrows (`&x`) can be taken (subject to no active exclusive borrows).
    *   An exclusive borrow (`&mut x`) can be taken (subject to no other active borrows at all).
    *   The binding `x` can also be reassigned (`x = new_value;`), which moves/copies `new_value` and drops the old value previously owned by `x`.
        ```ferra
        var s_mutable_path = String::from("changeable");
        let r_mut = &mut s_mutable_path; // OK
        *r_mut = String::from("changed");
        // r_mut's lifetime ends

        let r_shared = &s_mutable_path; // OK
        println(*r_shared);
        // r_shared's lifetime ends

        s_mutable_path = String::from("reassigned"); // OK: `var` binding allows reassignment.
        println(s_mutable_path);
        ```

This section provides a foundational understanding. The interaction with method receiver types (`self`, `&self`, `&mut self`) will further build upon these rules, typically defined on `behavior` or `data` type definitions.

## 3. Integration with Concurrency Model (Actor Model & Async)

Ensuring Ferra's ownership and borrowing rules work seamlessly with its concurrency model is paramount for achieving overall system safety and enabling developers to write robust concurrent applications. This section details these interactions, referencing `CONCURRENCY_MODEL.md`.

### 3.1. Ownership in Actor State

*   **State Isolation and Ownership**: As defined in `CONCURRENCY_MODEL.md` (Section 3.1), each actor instance exclusively owns its state (an instance of a Ferra `data` type). This state is not directly shared with other actors.
    ```ferra
    // From CONCURRENCY_MODEL.md
    data CounterState { count: Int }

    actor CounterBehavior {
        fn init(start_val: Int) -> CounterState {
            // `CounterState` instance is created and owned by the actor upon spawning.
            return CounterState { count: start_val };
        }

        // `self_state` is passed by value (moved in, new state moved out).
        async fn handle_increment(self_state: CounterState, _message: Increment) -> CounterState {
            let new_count = self_state.count + 1;
            // A new `CounterState` instance is returned, transferring ownership back to the actor's runtime.
            return CounterState { count: new_count };
        }
        // ... other handlers ...
    }
    ```
*   **State Transitions via Value Passing**: Actor message handlers (`async fn handle_...`) receive the current actor state by value (move semantics) and are expected to return the new state, also by value. This functional approach to state updates naturally upholds ownership boundaries, as the handler operates on an owned copy and produces a new owned state. There are no complex borrows of the actor's primary state across message handler invocations.
*   **No Direct Borrowing of Actor State Externally**: External entities (other actors or non-actor code) cannot directly borrow an actor's internal state. Communication and state observation happen exclusively through message passing.

### 3.2. Message Passing Semantics and Borrowing

*   **Messages are Owned Data**: As specified in `CONCURRENCY_MODEL.md` (Section 7.1, implicitly, and detailed in `OWNERSHIP_MODEL_PRINCIPLES_V0.1.md`), messages sent between actors are owned data. When a message is sent (e.g., `actor_ref ! MyMessage {...}` or `channel_sender.send(MyMessage {...})`), ownership of the message is moved from the sender to the communication mechanism (actor mailbox or channel buffer) and then to the receiving actor/task.
*   **No Active Borrows in Messages**: Consequently, messages generally cannot contain active references (borrows like `&T` or `&mut T`) that point to data owned by the sender's scope if that scope might end before the receiver processes the message. Doing so would lead to dangling references.
    *   If a reference needs to be shared, the data it points to must have a lifetime that is guaranteed to outlive the message's processing (e.g., static data, or data within an `Arc`-like shared ownership structure - though `Arc` itself is TBD for Ferra).
    *   Typically, any data needed by the receiver should be cloned or fully owned by the message itself.
    ```ferra
    data MyMessage { payload: String } // String is owned

    fn sender_actor_logic(recipient: ActorRef<MyActorBehavior>) {
        let local_string = String::from("data for message");
        // recipient ! MyMessage { payload: &local_string }; // ERROR: `local_string` might be dropped before recipient processes.
        recipient ! MyMessage { payload: local_string }; // OK: `local_string` is moved into the message.
    }
    ```
*   **`ActorRef<BehaviorType>` Ownership**: An `ActorRef` is a lightweight, shareable handle. `ActorRef` instances themselves are `Copy`-able (as per `OWNERSHIP_MODEL_PRINCIPLES_V0.1.md`, Section 2.3, for reference-like types). They do not own the actor they refer to; the actor system owns the actor instances. Multiple `ActorRef`s can point to the same actor.

### 3.3. Borrowing Across `await` Points

The introduction of `async`/`await` allows tasks (like actor message handlers) to suspend and resume. This poses challenges for borrows that might span an `await` point, as the state of the program can change during suspension. `CONCURRENCY_MODEL.md` (Section 4.3 and 7.3) touches upon this.

*   **Compiler Responsibility**: The Ferra compiler's borrow checker must ensure that any reference live across an `await` point remains valid when the task resumes.
    *   The data pointed to by the reference must not be dropped or mutably borrowed by another task in a way that invalidates the suspended task's reference.
*   **`Send` and `Sync` Concepts (Implicit for v0.1 Single-Threaded Scheduler)**:
    *   In a multi-threaded environment, types held across `await` points often need to be `Send` (if ownership is transferred to another thread for resumption) or the references themselves `Sync` (if data is shared and accessed by multiple threads).
    *   For Ferra's v0.1 deterministic single-scheduler model, these constraints are simpler. However, the principle of ensuring data validity and preventing data races across suspension/resumption points remains. The borrow checker, in conjunction with the actor model's state isolation, handles this.
*   **Restrictions on Borrows Across `await`**:
    *   **Mutable Borrows**: A mutable borrow (`&mut T`) generally cannot be held across an `await` point if the borrowed data could be accessed or modified by other concurrent operations during the suspension. This is because the exclusive access guarantee of `&mut T` would be violated.
        *   _Exception_: If the `&mut T` points to data that is exclusively owned by the current task and cannot be reached by other tasks (e.g., a local variable not captured by closures sent elsewhere), it might be permissible.
    *   **Shared Borrows**: Shared borrows (`&T`) held across an `await` are generally safer, provided the data they point to is not mutably aliased and modified by another task during suspension.
    ```ferra
    async fn example_await_borrow(mut owned_data: MyData, shared_global_data: &GlobalData) -> () {
        let local_ref = &owned_data; // Borrows data owned by this async task's scope.

        // await some_io_operation(); // Suspension point

        // `local_ref` is still used here.
        // This is generally OK if `owned_data` is not modified by other means during await.
        // `shared_global_data` must also remain valid and unchanged if it was borrowed immutably.
        println(local_ref.field);
        println(shared_global_data.info);

        //var mut_val = 10;
        //let r_mut = &mut mut_val;
        //await some_io_operation(); // If `mut_val` could be accessed elsewhere, holding `r_mut` is problematic.
                                  // For purely local stack data not shared, this might be fine.
    }
    ```
*   **Lifetime Extension**: The compiler must ensure that the lifetimes of borrowed data are correctly managed across `await` points. If an `async fn` borrows data from its arguments, those arguments must live at least as long as the entire `async fn` execution (including all suspensions and resumptions). This is related to higher-ranked trait bounds or equivalent concepts for `async fn` signatures, which are advanced topics.

### 3.4. Channels and Ownership/Borrowing

As detailed in `CONCURRENCY_MODEL.md` (Section 5.5):

*   **Move Semantics for Channel Data**: When data is sent via `channel_sender.send(message: T)`, ownership of `message` is moved into the channel. When `channel_receiver.receive()` yields `Ok(message: T)`, ownership is moved out to the receiver.
*   **Borrowing Before Send**: Before sending a value into a channel, normal borrowing rules apply. If you have a reference to a value, you cannot send the value itself if that would invalidate the reference (unless the reference's lifetime ends).
*   **Borrowing After Receive**: After receiving a value from a channel, the receiver owns it, and normal borrowing rules apply to this newly owned data.
*   **No Borrows of Channel Internals**: Users cannot borrow data that is "inside" the channel's buffer. Data is either owned by the sender (before send), the channel (during transit), or the receiver (after receive).

This strict ownership transfer through channels ensures memory safety and prevents data races, especially in systems that might later evolve to use true parallelism.

## 4. "Positive-First" Error Messaging & UX

An exceptional User Experience (UX) for Ferra's borrow checker is a primary design goal. This means diagnostics should not only be accurate but also highly understandable and actionable, guiding the developer towards correct code. This philosophy is directly inspired by the "positive-first" error messaging principle outlined in `Steps.md` (Section 3).

### 4.1. Design Principles for Borrow Checker Diagnostics

1.  **Positive Framing First**: As per `Steps.md`, diagnostics should strive to "emit *what is valid* before what is wrong." For example, state the valid lifetime or scope of a borrow before pointing out the conflict.
2.  **Clarity and Precision**: Messages must clearly identify the exact location (file, line, column span) of the error and the relevant entities (variables, references, types).
3.  **Actionable Hints**: Provide concrete, simple suggestions for resolving the error. These hints should be practical and consider common coding patterns.
4.  **Contextual Information**: Where helpful, provide context about the state of relevant variables or borrows that contribute to the error.
5.  **Avoid Jargon Where Possible**: While borrow checking involves specific terminology, explanations should aim for simplicity and avoid overly academic language.
6.  **Consistency**: Error message structure and terminology should be consistent across different types of borrow checking errors, aligning with `DESIGN_DIAGNOSTICS.md`.

### 4.2. Common Error Scenarios and Message Structure

Below are examples of common borrow checking errors and the desired structure for their diagnostic messages. Each diagnostic will be associated with a unique code from `diagnostic_codes.md` (e.g., `E400`). These conceptual outputs align with the structured diagnostic fields (such as `error_code`, `message`, `spans`, `labels`, and `help`) detailed in `DESIGN_DIAGNOSTICS.md`.

**Scenario 1: Conflicting Borrows (e.g., Mutable Borrow While Shared Borrows Exist)**

*   **Code Example**:
    ```ferra
    var data = String::from("example");
    let r1 = &data;         // Shared borrow starts
    let r2 = &data;         // Another shared borrow starts
    let r_mut = &mut data;  // ERROR: Tries to mutably borrow while r1 and r2 are live
    println(*r1);           // r1 used here
    ```
*   **Conceptual Diagnostic Output**:
    ```text
    error: cannot borrow `data` as mutable because it is already borrowed as immutable [E400]
      --> main.ferra:4:17
        |
    2   |     let r1 = &data;         // Shared borrow of `data` starts here
        |              ----- immutable borrow occurs here
    3   |     let r2 = &data;         // Another shared borrow of `data`
        |              ----- and another immutable borrow occurs here
    4   |     let r_mut = &mut data;  // Attempted mutable borrow
        |                 ^^^^^^^^^ mutable borrow occurs here
    5   |     println(*r1);           // Immutable borrow `r1` used here, extending its lifetime
        |
    ✔ `data` is immutably borrowed by `r1` (line 2) and `r2` (line 3).
      The borrow by `r1` is valid until at least line 5.
    ✘ A mutable borrow of `data` occurs at line 4, which conflicts with the active immutable borrows.

    Hint: To allow mutation of `data` at line 4, ensure all immutable borrows (like `r1`, `r2`) 
          are no longer live. This might involve ending their use or scope before line 4.
          Consider if `data` needs to be cloned if simultaneous independent access is required.
    ```

**Scenario 2: Use of Moved Value**

*   **Code Example**:
    ```ferra
    let s1 = String::from("hello");
    let s2 = s1;          // Ownership of string data moved from s1 to s2
    println(s1);         // ERROR: s1 was moved and is no longer valid
    ```
*   **Conceptual Diagnostic Output** (Ownership error, but related as borrow checker validates path validity):
    ```text
    error: use of moved value: `s1` [E401]
      --> main.ferra:3:13
        |
    1   |     let s1 = String::from("hello");
        |         -- value originally owned by `s1`
    2   |     let s2 = s1;          // `s1` moved here
        |              -- value moved to `s2` here
    3   |     println(s1);         // `s1` used after move
        |             ^^ `s1` is used here but its value was moved

    ✔ Value was moved to `s2` at line 2.
    ✘ `s1` cannot be used at line 3 because its ownership was transferred.

    Hint: If you need to use the value in multiple places, consider cloning it before the move:
          `let s2 = s1.clone();` or borrow `s1` using `&s1` if read-only access is sufficient.
    ```

**Scenario 3: Dangling Reference (Lifetime Error)**

*   **Code Example**:
    ```ferra
    let r: &String;
    {
        let s = String::from("scoped"); // s is owned by this inner scope
        r = &s;                        // r borrows s
    } // s is dropped here. r now refers to deallocated memory.
    println(*r);                       // ERROR: r is used, but s is out of scope
    ```
*   **Conceptual Diagnostic Output**:
    ```text
    error: `s` does not live long enough for this borrow [E402]
      --> main.ferra:5:15
        |
    2   |     {
    3   |         let s = String::from("scoped"); // `s` is defined here
    4   |         r = &s;                        // Borrow of `s` occurs here...
        |             -- ...and is assigned to `r` whose lifetime extends beyond this scope
    5   |     } // `s` is dropped here, but it is still borrowed by `r`
        |     - `s` dropped here while still borrowed
    6   |     println(*r);                       // Borrowed value `r` is used here.
        |              ^ `r`'s borrow needs `s` to live until here

    ✔ `s` is valid within the scope defined at lines 2-5.
    ✘ The reference `r` (borrowing `s`) is used at line 6, but `s` was already dropped at the end of its scope on line 5.

    Hint: To fix this, ensure `s` lives as long as `r` needs it. This might involve:
          1. Moving the definition of `s` to an outer scope that encloses line 6.
          2. Returning ownership of the data from the inner scope if `r` needs to own it.
          3. Restructuring the code so `r` is only used while `s` is valid.
    ```

### 4.3. Visual Structure of Diagnostics in IDEs (Conceptual)

To enhance understandability, IDE integrations (e.g., via the VSCode plugin `VSCODE_PLUGIN_ALPHA_SPEC.md`) should aim for rich visual feedback:

*   **Color Coding**: Different colors for valid borrow regions, conflicting borrow attempts, and hints.
*   **Inline Annotations**: Displaying brief error messages or hints directly in the code editor near the erroneous line.
*   **Gutter Icons**: Using icons in the editor gutter to indicate error locations and severity.
*   **Hover Information**: Providing detailed diagnostic messages and suggestions when the user hovers over an error or a relevant variable/reference.
*   **Arrows and Underlines**: Using graphical elements like arrows to point from borrows to their origins or to highlight conflicting regions, as suggested in the text examples above.

### 4.4. Cross-Referencing with Diagnostics System

*   **Diagnostic Codes**: All borrow checker errors will have unique codes (e.g., `FERR_BORROW_XXX`, `FERR_LIFETIME_XXX`) registered in `diagnostic_codes.md`. This aids in documentation, searching for solutions, and potentially for `ai::explain(err)` integration.
*   **Error Spans**: Diagnostics will include precise start and end locations (spans) for errors, references, and relevant variable definitions, as per `DESIGN_DIAGNOSTICS.md`.
*   **Severity**: Borrow checking errors are typically hard errors (preventing compilation), but the diagnostics system might allow for future warnings on more subtle or stylistic ownership issues.
*   **Consistency**: The structure and presentation of borrow checker diagnostics will adhere to the general guidelines established in `DESIGN_DIAGNOSTICS.md` for all compiler diagnostics.

## 5. Integration of AI Tags for Borrow Checker

Ferra aims to be an AI-Native language, and one aspect of this is allowing AI tools (and expert developers) to provide hints or make assertions that can influence compiler behavior, particularly in areas like borrow checking for complex or performance-critical code. This section details the integration of such AI tags, focusing on the `ai.assume(nll="noalias")` tag mentioned in `Steps.md` (Section 2).

### 5.1. The `ai.assume(nll="noalias")` Tag

*   **Purpose**: The primary purpose of the `ai.assume(nll="noalias")` tag is to allow the developer or an AI-driven refactoring tool to assert to the borrow checker that a particular reference (or pointer, especially in FFI contexts) does not alias with any other references/pointers within a certain scope or under specific conditions. This can help resolve complex borrow checking scenarios that the compiler might conservatively reject, or to enable optimizations that rely on non-aliasing guarantees.
    *   This is particularly relevant for Non-Lexical Lifetimes (NLL) analysis, where understanding aliasing is crucial for determining how long borrows truly need to be live.
    *   It can also be vital when interfacing with C/C++ code via FFI, where the Ferra compiler has limited visibility into the behavior of external code.
*   **Syntax and Placement**: The `ai.assume(...)` tag is envisioned as an attribute-like annotation that can be applied to variable bindings, function parameters, or potentially specific expressions.
    ```ferra
    // Conceptual syntax examples

    // Applied to a variable binding (e.g., a raw pointer from FFI)
    #[ai.assume(nll="noalias")]
    let raw_ptr: *mut u8 = get_raw_pointer_from_c();

    // Applied to a function parameter
    fn process_data(#[ai.assume(nll="noalias")] data_slice: &mut [u8]) {
        // ... compiler can assume data_slice does not alias other pointers accessible here ...
    }

    // Potentially on an expression, though less common for `noalias`
    // let result = (some_expression #[ai.assume(nll="noalias")]).do_something();
    ```
    The precise syntax will need to be harmonized with Ferra's general attribute/annotation syntax defined in `SYNTAX_GRAMMAR_V0.1.md`.
*   **Source of the Tag**: While developers can manually insert these tags, a key aspect of Ferra's AI-Native design is that AI-powered static analysis or refactoring tools could suggest or automatically insert these tags based on a deeper understanding of the code or specific patterns.

### 5.2. Semantics and Impact on Borrow Checking

*   **Compiler Interpretation**: When the Ferra borrow checker encounters an `ai.assume(nll="noalias")` tag associated with a reference/pointer `p`:
    *   It will trust this assertion and, for the scope of `p`'s validity (or as further refined by the NLL analysis informed by this tag), assume that `p` provides exclusive access to the memory region it points to, relative to other pointers that might otherwise be considered ambiguous by the checker.
    *   This can allow the borrow checker to accept code it would otherwise reject due to potential aliasing conflicts, especially if NLL analysis relies on proving non-overlap of borrow regions.
*   **Example Scenario**: Consider a function that takes two mutable slices that might or might not overlap.
    ```ferra
    fn process_two_slices(s1: &mut [Int], s2: &mut [Int]) {
        // Without `noalias`, compiler might be conservative if s1 and s2 could alias.
        // If they could alias, modifying s1 could affect s2 in unexpected ways.
        s1[0] = 10;
        s2[0] = 20; // If s1 and s2 point to the same memory, s1[0] is now 20.
    }

    // With `noalias` assertion if the caller guarantees non-overlap:
    fn process_two_slices_optimized(
        #[ai.assume(nll="noalias")] s1: &mut [Int], 
        #[ai.assume(nll="noalias")] s2: &mut [Int]
        // Optional: further assume s1 and s2 themselves don't overlap, e.g. ai.assume(disjoint(s1,s2))
    ) {
        // Compiler can be more aggressive with optimizations or borrow relaxations.
        s1[0] = 10;
        s2[0] = 20;
    }
    ```
*   **Not a General `unsafe` Block**: This tag is more targeted than a general `unsafe` block. It provides a specific assertion to the borrow checker, rather than disabling all safety checks for a region of code. However, its misuse can lead to undefined behavior.

### 5.3. Risks and Developer Responsibility

*   **Unsoundness if Misused**: If `ai.assume(nll="noalias")` is applied incorrectly (i.e., the asserted non-aliasing condition is false), it can lead to data races, memory corruption, or other undefined behavior, as the borrow checker will operate on a false premise. This effectively bypasses a crucial safety check.
*   **Developer/Tool Responsibility**: The ultimate responsibility for the correctness of an `ai.assume` tag lies with the entity that places it (the developer or the AI tool). It signifies an assertion that the Ferra compiler itself cannot statically prove but is instructed to trust.
*   **Debugging Challenges**: Incorrect `noalias` assumptions can make debugging significantly harder, as the observed behavior might contradict the compiler's (instructed) understanding of memory access patterns.
*   **Use Sparingly**: These tags should be used sparingly and only when necessary, typically for FFI, highly optimized low-level code, or when an AI tool has high confidence in the assertion based on advanced program analysis.

### 5.4. Other Potential AI-driven Borrow Checking Assumptions (Future Considerations)

Beyond `noalias`, one could envision other `ai.assume` tags relevant to ownership and borrowing in the future:

*   **`ai.assume(lifetime_is='static)`**: Asserting that a particular reference has a static lifetime, even if not provable by standard inference (e.g., for data from FFI known to be static).
*   **`ai.assume(disjoint(ref1, ref2))`**: Explicitly asserting that two references, `ref1` and `ref2`, point to entirely disjoint memory regions.
*   **`ai.assume(initialized)`**: Asserting that a piece of memory is initialized, bypassing compiler checks for uninitialized reads (extremely unsafe if wrong).

These would all carry similar risks and responsibilities. Their specification would require careful consideration of use cases and potential for unsoundness.

## 6. Open Questions / TBD

*   (TAG-BORROW-1) Detailed strategy for Non-Lexical Lifetimes (NLL) visualization or explanation in errors.
*   (TAG-BORROW-2) Specifics of borrow checking in pattern matching and destructuring.
*   (TAG-BORROW-3) Interaction with more advanced type system features (e.g., generics, traits if they have borrowing implications).
*   (TAG-BORROW-4) Formal specification of explicit lifetime syntax (e.g., `'a`) and rules for their elision/inference in function signatures and data structures.
*   (TAG-BORROW-5) Detailed rules for borrow checking `self`, `&self`, and `&mut self` in method definitions and calls, including their interaction with `data` and `actor behavior` types.

---
This document will define the refined borrow checking mechanism for Ferra, its integration with the concurrency model, its user experience goals for error reporting, and how AI-provided hints can be incorporated. 