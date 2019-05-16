/// General Structure of an instruction.
pub struct Inst {}

/// Enumeration of different Instruction Kinds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstKind {
    /// Op ///
    read,
    end,
    writeNL,
    kill,

    /// Op x ///
    neg,
    write,
    ret,

    /// Op x y ///
    add,
    sub,
    mul,
    div,
    cmp,
    adda,

    bne,
    beq,
    ble,
    blt,
    bge,
    bgt,

    phi,

    /// Op y ///
    load,
    loadsp,
    pload,
    gload,
    bra,

    /// Op y x ///
    store,
    mov,

    spill,

    // Indicate that function store register
    // value for function parameter.
    // Same layout as store.
    // param (x) location (y) value
    pstore,

    // Indicate that function store register value
    // for a global affected within the function.
    // Same layout as store.
    // global (x) location (y) value
    gstore,

    /// Op Str ///
    call,
}