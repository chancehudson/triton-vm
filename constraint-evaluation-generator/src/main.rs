use std::collections::HashSet;
use std::process::Command;

use itertools::Itertools;
use twenty_first::shared_math::b_field_element::BFieldElement;
use twenty_first::shared_math::x_field_element::XFieldElement;

use triton_vm::table::cascade_table::ExtCascadeTable;
use triton_vm::table::constraint_circuit::CircuitExpression;
use triton_vm::table::constraint_circuit::ConstraintCircuit;
use triton_vm::table::constraint_circuit::ConstraintCircuitBuilder;
use triton_vm::table::constraint_circuit::ConstraintCircuitMonad;
use triton_vm::table::constraint_circuit::InputIndicator;
use triton_vm::table::cross_table_argument::GrandCrossTableArg;
use triton_vm::table::hash_table::ExtHashTable;
use triton_vm::table::jump_stack_table::ExtJumpStackTable;
use triton_vm::table::lookup_table::ExtLookupTable;
use triton_vm::table::op_stack_table::ExtOpStackTable;
use triton_vm::table::processor_table::ExtProcessorTable;
use triton_vm::table::program_table::ExtProgramTable;
use triton_vm::table::ram_table::ExtRamTable;
use triton_vm::table::u32_table::ExtU32Table;

fn main() {
    let (table_name_snake, table_name_camel) = construct_needed_table_identifiers(&["program"]);
    let source_code = gen(
        &table_name_snake,
        &table_name_camel,
        &mut build_fold_circuitify(&ExtProgramTable::initial_constraints),
        &mut build_fold_circuitify(&ExtProgramTable::consistency_constraints),
        &mut build_fold_circuitify(&ExtProgramTable::transition_constraints),
        &mut build_fold_circuitify(&ExtProgramTable::terminal_constraints),
    );
    write(&table_name_snake, source_code);

    let (table_name_snake, table_name_camel) = construct_needed_table_identifiers(&["processor"]);
    let source_code = gen(
        &table_name_snake,
        &table_name_camel,
        &mut build_fold_circuitify(&ExtProcessorTable::initial_constraints),
        &mut build_fold_circuitify(&ExtProcessorTable::consistency_constraints),
        &mut build_fold_circuitify(&ExtProcessorTable::transition_constraints),
        &mut build_fold_circuitify(&ExtProcessorTable::terminal_constraints),
    );
    write(&table_name_snake, source_code);

    let (table_name_snake, table_name_camel) = construct_needed_table_identifiers(&["op", "stack"]);
    let source_code = gen(
        &table_name_snake,
        &table_name_camel,
        &mut build_fold_circuitify(&ExtOpStackTable::initial_constraints),
        &mut build_fold_circuitify(&ExtOpStackTable::consistency_constraints),
        &mut build_fold_circuitify(&ExtOpStackTable::transition_constraints),
        &mut build_fold_circuitify(&ExtOpStackTable::terminal_constraints),
    );
    write(&table_name_snake, source_code);

    let (table_name_snake, table_name_camel) = construct_needed_table_identifiers(&["ram"]);
    let source_code = gen(
        &table_name_snake,
        &table_name_camel,
        &mut build_fold_circuitify(&ExtRamTable::initial_constraints),
        &mut build_fold_circuitify(&ExtRamTable::consistency_constraints),
        &mut build_fold_circuitify(&ExtRamTable::transition_constraints),
        &mut build_fold_circuitify(&ExtRamTable::terminal_constraints),
    );
    write(&table_name_snake, source_code);

    let (table_name_snake, table_name_camel) =
        construct_needed_table_identifiers(&["jump", "stack"]);
    let source_code = gen(
        &table_name_snake,
        &table_name_camel,
        &mut build_fold_circuitify(&ExtJumpStackTable::initial_constraints),
        &mut build_fold_circuitify(&ExtJumpStackTable::consistency_constraints),
        &mut build_fold_circuitify(&ExtJumpStackTable::transition_constraints),
        &mut build_fold_circuitify(&ExtJumpStackTable::terminal_constraints),
    );
    write(&table_name_snake, source_code);

    let (table_name_snake, table_name_camel) = construct_needed_table_identifiers(&["hash"]);
    let source_code = gen(
        &table_name_snake,
        &table_name_camel,
        &mut build_fold_circuitify(&ExtHashTable::initial_constraints),
        &mut build_fold_circuitify(&ExtHashTable::consistency_constraints),
        &mut build_fold_circuitify(&ExtHashTable::transition_constraints),
        &mut build_fold_circuitify(&ExtHashTable::terminal_constraints),
    );
    write(&table_name_snake, source_code);

    let (table_name_snake, table_name_camel) = construct_needed_table_identifiers(&["cascade"]);
    let source_code = gen(
        &table_name_snake,
        &table_name_camel,
        &mut build_fold_circuitify(&ExtCascadeTable::initial_constraints),
        &mut build_fold_circuitify(&ExtCascadeTable::consistency_constraints),
        &mut build_fold_circuitify(&ExtCascadeTable::transition_constraints),
        &mut build_fold_circuitify(&ExtCascadeTable::terminal_constraints),
    );
    write(&table_name_snake, source_code);

    let (table_name_snake, table_name_camel) = construct_needed_table_identifiers(&["lookup"]);
    let source_code = gen(
        &table_name_snake,
        &table_name_camel,
        &mut build_fold_circuitify(&ExtLookupTable::initial_constraints),
        &mut build_fold_circuitify(&ExtLookupTable::consistency_constraints),
        &mut build_fold_circuitify(&ExtLookupTable::transition_constraints),
        &mut build_fold_circuitify(&ExtLookupTable::terminal_constraints),
    );
    write(&table_name_snake, source_code);

    let (table_name_snake, table_name_camel) = construct_needed_table_identifiers(&["u32"]);
    let source_code = gen(
        &table_name_snake,
        &table_name_camel,
        &mut build_fold_circuitify(&ExtU32Table::initial_constraints),
        &mut build_fold_circuitify(&ExtU32Table::consistency_constraints),
        &mut build_fold_circuitify(&ExtU32Table::transition_constraints),
        &mut build_fold_circuitify(&ExtU32Table::terminal_constraints),
    );
    write(&table_name_snake, source_code);

    let table_name_snake = "cross_table_argument";
    let table_name_camel = "GrandCrossTableArg";
    let source_code = gen(
        table_name_snake,
        table_name_camel,
        &mut build_fold_circuitify(&GrandCrossTableArg::initial_constraints),
        &mut build_fold_circuitify(&GrandCrossTableArg::consistency_constraints),
        &mut build_fold_circuitify(&GrandCrossTableArg::transition_constraints),
        &mut build_fold_circuitify(&GrandCrossTableArg::terminal_constraints),
    );
    write(table_name_snake, source_code);

    if let Err(fmt_failed) = Command::new("cargo").arg("fmt").output() {
        println!("cargo fmt failed: {fmt_failed}");
    }
}

/// Get the constraints defined in the given function, perform constant folding, and return
/// them as a vector of `ConstraintCircuit`s.
fn build_fold_circuitify<II: InputIndicator>(
    circuit_monad_function: &dyn Fn(
        &ConstraintCircuitBuilder<II>,
    ) -> Vec<ConstraintCircuitMonad<II>>,
) -> Vec<ConstraintCircuit<II>> {
    let circuit_builder = ConstraintCircuitBuilder::new();
    let mut constraints = circuit_monad_function(&circuit_builder);
    ConstraintCircuitMonad::constant_folding(&mut constraints);
    constraints
        .into_iter()
        .map(|circuit| circuit.consume())
        .collect()
}

fn construct_needed_table_identifiers(table_name_constituents: &[&str]) -> (String, String) {
    let table_name_snake = format!("{}_table", table_name_constituents.join("_"));
    let title_case = table_name_constituents
        .iter()
        .map(|part| {
            let (first_char, rest) = part.split_at(1);
            let first_char_upper = first_char.to_uppercase();
            format!("{first_char_upper}{rest}")
        })
        .collect_vec();
    let table_name_camel = format!("Ext{}Table", title_case.iter().join(""));
    (table_name_snake, table_name_camel)
}

fn write(table_name_snake: &str, rust_source_code: String) {
    let output_filename =
        format!("triton-vm/src/table/constraints/{table_name_snake}_constraints.rs");

    std::fs::write(output_filename, rust_source_code).expect("Write Rust source code");
}

fn gen<SII: InputIndicator, DII: InputIndicator>(
    table_name_snake: &str,
    table_mod_name: &str,
    initial_constraint_circuits: &mut [ConstraintCircuit<SII>],
    consistency_constraint_circuits: &mut [ConstraintCircuit<SII>],
    transition_constraint_circuits: &mut [ConstraintCircuit<DII>],
    terminal_constraint_circuits: &mut [ConstraintCircuit<SII>],
) -> String {
    let num_initial_constraints = initial_constraint_circuits.len();
    let num_consistency_constraints = consistency_constraint_circuits.len();
    let num_transition_constraints = transition_constraint_circuits.len();
    let num_terminal_constraints = terminal_constraint_circuits.len();

    let (
        initial_constraints_degrees,
        initial_constraint_strings_bfe,
        initial_constraint_strings_xfe,
    ) = turn_circuits_into_string(initial_constraint_circuits);
    let (
        consistency_constraints_degrees,
        consistency_constraint_strings_bfe,
        consistency_constraint_strings_xfe,
    ) = turn_circuits_into_string(consistency_constraint_circuits);
    let (
        transition_constraints_degrees,
        transition_constraint_strings_bfe,
        transition_constraint_strings_xfe,
    ) = turn_circuits_into_string(transition_constraint_circuits);
    let (
        terminal_constraints_degrees,
        terminal_constraint_strings_bfe,
        terminal_constraint_strings_xfe,
    ) = turn_circuits_into_string(terminal_constraint_circuits);

    format!(
        "
use ndarray::ArrayView1;
use twenty_first::shared_math::b_field_element::BFieldElement;
use twenty_first::shared_math::mpolynomial::Degree;
use twenty_first::shared_math::x_field_element::XFieldElement;

use crate::table::challenges::Challenges;
use crate::table::challenges::ChallengeId::*;
use crate::table::extension_table::Evaluable;
use crate::table::extension_table::Quotientable;
use crate::table::{table_name_snake}::{table_mod_name};

// This file has been auto-generated. Any modifications _will_ be lost.
// To re-generate, execute:
// `cargo run --bin constraint-evaluation-generator`
impl Evaluable<BFieldElement> for {table_mod_name} {{
    #[inline]
    #[allow(unused_variables)]
    fn evaluate_initial_constraints(
        base_row: ArrayView1<BFieldElement>,
        ext_row: ArrayView1<XFieldElement>,
        challenges: &Challenges,
    ) -> Vec<XFieldElement> {{
        {initial_constraint_strings_bfe}
    }}

    #[inline]
    #[allow(unused_variables)]
    fn evaluate_consistency_constraints(
        base_row: ArrayView1<BFieldElement>,
        ext_row: ArrayView1<XFieldElement>,
        challenges: &Challenges,
    ) -> Vec<XFieldElement> {{
        {consistency_constraint_strings_bfe}
    }}

    #[inline]
    #[allow(unused_variables)]
    fn evaluate_transition_constraints(
        current_base_row: ArrayView1<BFieldElement>,
        current_ext_row: ArrayView1<XFieldElement>,
        next_base_row: ArrayView1<BFieldElement>,
        next_ext_row: ArrayView1<XFieldElement>,
        challenges: &Challenges,
    ) -> Vec<XFieldElement> {{
        {transition_constraint_strings_bfe}
    }}

    #[inline]
    #[allow(unused_variables)]
    fn evaluate_terminal_constraints(
        base_row: ArrayView1<BFieldElement>,
        ext_row: ArrayView1<XFieldElement>,
        challenges: &Challenges,
    ) -> Vec<XFieldElement> {{
        {terminal_constraint_strings_bfe}
    }}
}}

impl Evaluable<XFieldElement> for {table_mod_name} {{
    #[inline]
    #[allow(unused_variables)]
    fn evaluate_initial_constraints(
        base_row: ArrayView1<XFieldElement>,
        ext_row: ArrayView1<XFieldElement>,
        challenges: &Challenges,
    ) -> Vec<XFieldElement> {{
        {initial_constraint_strings_xfe}
    }}

    #[inline]
    #[allow(unused_variables)]
    fn evaluate_consistency_constraints(
        base_row: ArrayView1<XFieldElement>,
        ext_row: ArrayView1<XFieldElement>,
        challenges: &Challenges,
    ) -> Vec<XFieldElement> {{
        {consistency_constraint_strings_xfe}
    }}

    #[inline]
    #[allow(unused_variables)]
    fn evaluate_transition_constraints(
        current_base_row: ArrayView1<XFieldElement>,
        current_ext_row: ArrayView1<XFieldElement>,
        next_base_row: ArrayView1<XFieldElement>,
        next_ext_row: ArrayView1<XFieldElement>,
        challenges: &Challenges,
    ) -> Vec<XFieldElement> {{
        {transition_constraint_strings_xfe}
    }}

    #[inline]
    #[allow(unused_variables)]
    fn evaluate_terminal_constraints(
        base_row: ArrayView1<XFieldElement>,
        ext_row: ArrayView1<XFieldElement>,
        challenges: &Challenges,
    ) -> Vec<XFieldElement> {{
        {terminal_constraint_strings_xfe}
    }}
}}

impl Quotientable for {table_mod_name} {{
    fn num_initial_quotients() -> usize {{
        {num_initial_constraints}
    }}

    fn num_consistency_quotients() -> usize {{
        {num_consistency_constraints}
    }}

    fn num_transition_quotients() -> usize {{
        {num_transition_constraints}
    }}

    fn num_terminal_quotients() -> usize {{
        {num_terminal_constraints}
    }}

    #[allow(unused_variables)]
    fn initial_quotient_degree_bounds(
        interpolant_degree: Degree,
    ) -> Vec<Degree> {{
        let zerofier_degree = 1;
        [{initial_constraints_degrees}].to_vec()
    }}

    #[allow(unused_variables)]
    fn consistency_quotient_degree_bounds(
        interpolant_degree: Degree,
        padded_height: usize,
    ) -> Vec<Degree> {{
        let zerofier_degree = padded_height as Degree;
        [{consistency_constraints_degrees}].to_vec()
    }}

    #[allow(unused_variables)]
    fn transition_quotient_degree_bounds(
        interpolant_degree: Degree,
        padded_height: usize,
    ) -> Vec<Degree> {{
        let zerofier_degree = padded_height as Degree - 1;
        [{transition_constraints_degrees}].to_vec()
    }}

    #[allow(unused_variables)]
    fn terminal_quotient_degree_bounds(
        interpolant_degree: Degree,
    ) -> Vec<Degree> {{
        let zerofier_degree = 1;
        [{terminal_constraints_degrees}].to_vec()
    }}
}}
"
    )
}

/// Given a slice of constraint circuits, return a tuple of strings corresponding to code
/// evaluating these constraints as well as their degrees. In particular:
/// 1. The first string contains code that, when evaluated, produces the constraints' degrees,
/// 1. the second string contains code that, when evaluated, produces the constraints' values, with
///     the input type for the base row being `BFieldElement`, and
/// 1. the third string is like the second string, except that the input type for the base row is
///    `XFieldElement`.
fn turn_circuits_into_string<II: InputIndicator>(
    constraint_circuits: &mut [ConstraintCircuit<II>],
) -> (String, String, String) {
    if constraint_circuits.is_empty() {
        return ("".to_string(), "vec![]".to_string(), "vec![]".to_string());
    }

    // Sanity check: all node IDs must be unique.
    ConstraintCircuit::assert_has_unique_ids(constraint_circuits);

    // Count number of times each node is referenced.
    ConstraintCircuit::traverse_multiple(constraint_circuits);

    // Get all unique reference counts.
    let mut visited_counters = constraint_circuits
        .iter()
        .flat_map(|constraint| constraint.get_all_visited_counters())
        .collect_vec();
    visited_counters.sort_unstable();
    visited_counters.dedup();

    // Declare all shared variables, i.e., those with a visit count greater than 1.
    // These declarations must be made starting from the highest visit count.
    // Otherwise, the resulting code will refer to bindings that have not yet been made.
    let shared_declarations = visited_counters
        .into_iter()
        .filter(|&x| x > 1)
        .rev()
        .map(|visit_count| declare_nodes_with_visit_count(visit_count, constraint_circuits))
        .collect_vec()
        .join("");

    let (base_constraints, ext_constraints): (Vec<_>, Vec<_>) = constraint_circuits
        .iter()
        .partition(|constraint| is_bfield_element(constraint));

    // The order of the constraints' degrees must match the order of the constraints.
    // Hence, listing the degrees is only possible after the partition into base and extension
    // constraints is known.
    let degree_bounds_string = base_constraints
        .iter()
        .chain(ext_constraints.iter())
        .map(|circuit| match circuit.degree() {
            d if d > 1 => format!("interpolant_degree * {d} - zerofier_degree"),
            d if d == 1 => "interpolant_degree - zerofier_degree".to_string(),
            _ => unreachable!("Constraint degree must be positive"),
        })
        .join(",\n");

    let build_constraint_evaluation_code = |constraints: &[&ConstraintCircuit<II>]| {
        constraints
            .iter()
            .map(|constraint| evaluate_single_node(1, constraint, &HashSet::default()))
            .collect_vec()
            .join(",\n")
    };
    let base_constraint_strings = build_constraint_evaluation_code(&base_constraints);
    let ext_constraint_strings = build_constraint_evaluation_code(&ext_constraints);

    // If there are no base constraints, the type needs to be explicitly declared.
    let base_constraint_bfe_type = match base_constraints.is_empty() {
        true => ": [BFieldElement; 0]",
        false => "",
    };

    let constraint_string_bfe = format!(
        "{shared_declarations}
        let base_constraints{base_constraint_bfe_type} = [{base_constraint_strings}];
        let ext_constraints = [{ext_constraint_strings}];
        base_constraints
            .into_iter()
            .map(|bfe| bfe.lift())
            .chain(ext_constraints.into_iter())
            .collect()"
    );

    let constraint_string_xfe = format!(
        "{shared_declarations}
        let base_constraints = [{base_constraint_strings}];
        let ext_constraints = [{ext_constraint_strings}];
        base_constraints
            .into_iter()
            .chain(ext_constraints.into_iter())
            .collect()"
    );

    (
        degree_bounds_string,
        constraint_string_bfe,
        constraint_string_xfe,
    )
}

/// Produce the code to evaluate code for all nodes that share a value number of
/// times visited. A value for all nodes with a higher count than the provided are assumed
/// to be in scope.
fn declare_nodes_with_visit_count<II: InputIndicator>(
    requested_visited_count: usize,
    circuits: &[ConstraintCircuit<II>],
) -> String {
    let mut scope: HashSet<usize> = HashSet::new();

    circuits
        .iter()
        .map(|circuit| {
            declare_single_node_with_visit_count(circuit, requested_visited_count, &mut scope)
        })
        .collect_vec()
        .join("")
}

fn declare_single_node_with_visit_count<II: InputIndicator>(
    circuit: &ConstraintCircuit<II>,
    requested_visited_count: usize,
    scope: &mut HashSet<usize>,
) -> String {
    // Don't declare a node twice.
    if scope.contains(&circuit.id) {
        return String::default();
    }

    // A higher-than-requested visit counter means the node is already in global scope, albeit not
    // necessarily in the passed-in scope.
    if circuit.visited_counter > requested_visited_count {
        return String::default();
    }

    let CircuitExpression::BinaryOperation(_, lhs, rhs) = &circuit.expression else {
        // Constants are already (or can be) trivially declared.
        return String::default();
    };

    // If the visited counter is not yet exact, start recursing on the BinaryOperation's children.
    if circuit.visited_counter < requested_visited_count {
        let out_left = declare_single_node_with_visit_count(
            &lhs.as_ref().borrow(),
            requested_visited_count,
            scope,
        );
        let out_right = declare_single_node_with_visit_count(
            &rhs.as_ref().borrow(),
            requested_visited_count,
            scope,
        );
        return [out_left, out_right].join("\n");
    }

    // Declare a new binding.
    assert_eq!(circuit.visited_counter, requested_visited_count);
    let binding_name = get_binding_name(circuit);
    let evaluation = evaluate_single_node(requested_visited_count, circuit, scope);

    let is_new_insertion = scope.insert(circuit.id);
    assert!(is_new_insertion);

    format!("let {binding_name} = {evaluation};\n")
}

/// Return a variable name for the node. Returns `point[n]` if node is just
/// a value from the codewords. Otherwise returns the ID of the circuit.
fn get_binding_name<II: InputIndicator>(circuit: &ConstraintCircuit<II>) -> String {
    match &circuit.expression {
        CircuitExpression::BConstant(bfe) => print_bfe(bfe),
        CircuitExpression::XConstant(xfe) => print_xfe(xfe),
        CircuitExpression::Input(idx) => idx.to_string(),
        CircuitExpression::Challenge(challenge_id) => {
            format!("challenges.get_challenge({challenge_id})")
        }
        CircuitExpression::BinaryOperation(_, _, _) => format!("node_{}", circuit.id),
    }
}

fn print_bfe(bfe: &BFieldElement) -> String {
    format!("BFieldElement::from_raw_u64({})", bfe.raw_u64())
}

fn print_xfe(xfe: &XFieldElement) -> String {
    let coeff_0 = print_bfe(&xfe.coefficients[0]);
    let coeff_1 = print_bfe(&xfe.coefficients[1]);
    let coeff_2 = print_bfe(&xfe.coefficients[2]);
    format!("XFieldElement::new([{coeff_0}, {coeff_1}, {coeff_2}])")
}

/// Recursively check whether a node is composed of only BFieldElements, i.e., only uses
/// 1. inputs from base rows,
/// 2. constants from the B-field, and
/// 3. binary operations on BFieldElements.
fn is_bfield_element<II: InputIndicator>(circuit: &ConstraintCircuit<II>) -> bool {
    match &circuit.expression {
        CircuitExpression::BConstant(_) => true,
        CircuitExpression::XConstant(_) => false,
        CircuitExpression::Input(indicator) => indicator.is_base_table_column(),
        CircuitExpression::Challenge(_) => false,
        CircuitExpression::BinaryOperation(_, lhs, rhs) => {
            let lhs = lhs.as_ref().borrow();
            let rhs = rhs.as_ref().borrow();
            is_bfield_element(&lhs) && is_bfield_element(&rhs)
        }
    }
}

/// Recursively construct the code for evaluating a single node.
fn evaluate_single_node<II: InputIndicator>(
    requested_visited_count: usize,
    circuit: &ConstraintCircuit<II>,
    scope: &HashSet<usize>,
) -> String {
    let binding_name = get_binding_name(circuit);

    // Don't declare a node twice.
    if scope.contains(&circuit.id) {
        return binding_name;
    }

    // The binding must already be known.
    if circuit.visited_counter > requested_visited_count {
        return binding_name;
    }

    // Constants have trivial bindings.
    let CircuitExpression::BinaryOperation(binop, lhs, rhs) = &circuit.expression else {
        return binding_name;
    };

    let lhs = lhs.as_ref().borrow();
    let rhs = rhs.as_ref().borrow();
    let evaluated_lhs = evaluate_single_node(requested_visited_count, &lhs, scope);
    let evaluated_rhs = evaluate_single_node(requested_visited_count, &rhs, scope);
    let operation = binop.to_string();
    format!("({evaluated_lhs}) {operation} ({evaluated_rhs})")
}
