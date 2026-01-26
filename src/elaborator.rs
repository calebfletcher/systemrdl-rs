#![allow(unused)]

use std::{collections::HashMap, hash::Hash};

use crate::ast::{self, PrimaryLiteral};

#[derive(Debug, Clone)]
pub struct RootNamespace {
    nodes: Vec<Node>,
}

pub fn elaborate(ast: ast::Root) -> Result<RootNamespace, anyhow::Error> {
    let mut root = RootNamespace { nodes: Vec::new() };

    for desc in ast.descriptions {
        match desc {
            ast::Description::ComponentDef(component) => {
                root.nodes.append(&mut elaborate_component(&component));
            }
            ast::Description::EnumDef(enum_def) => todo!(),
            ast::Description::PropertyDefinition(property_definition) => todo!(),
            ast::Description::StructDef(struct_def) => todo!(),
            ast::Description::ConstraintDef(constraint_def) => todo!(),
            ast::Description::ExplicitComponentInst(explicit_component_inst) => todo!(),
            ast::Description::PropertyAssignment(property_assignment) => todo!(),
        }
    }

    Ok(root)
}

fn elaborate_component(component: &ast::Component) -> Vec<Node> {
    assert!(component.inst_type.is_none());

    if let Some(insts) = &component.insts {
        assert!(insts.param_insts.is_empty());

        let mut result = Vec::new();

        for inst in &insts.component_insts {
            //assert!(inst.array_or_range.is_none());
            let _at = inst.at.as_ref().map(evaluate_constants);
            //assert!(inst.equals.is_none());
            assert!(inst.percent_equals.is_none());
            assert!(inst.plus_equals.is_none());

            let node = match &component.def {
                ast::ComponentDef::Named(component_type, _, param_def, component_body) => todo!(),
                ast::ComponentDef::Anon(component_type, component_body) => {
                    //
                    match component_type {
                        ast::ComponentType::Field => {
                            elaborate_field(inst.id.clone(), component_body)
                        }
                        ast::ComponentType::Reg => elaborate_reg(inst.id.clone(), component_body),
                        ast::ComponentType::RegFile => todo!(),
                        ast::ComponentType::AddrMap => {
                            elaborate_addrmap(inst.id.clone(), component_body)
                        }
                        ast::ComponentType::Signal => todo!(),
                        ast::ComponentType::Enum => todo!(),
                        ast::ComponentType::EnumVariant => todo!(),
                        ast::ComponentType::Mem => todo!(),
                        ast::ComponentType::Constraint => todo!(),
                    }
                }
            };
            result.push(node);
        }
        result
    } else {
        let node = match &component.def {
            ast::ComponentDef::Named(component_type, name, param_def, component_body) => {
                match component_type {
                    ast::ComponentType::Field => todo!(),
                    ast::ComponentType::Reg => todo!(),
                    ast::ComponentType::RegFile => todo!(),
                    ast::ComponentType::AddrMap => {
                        assert!(param_def.is_none());
                        elaborate_addrmap(name.clone(), component_body)
                    }
                    ast::ComponentType::Signal => todo!(),
                    ast::ComponentType::Enum => todo!(),
                    ast::ComponentType::EnumVariant => todo!(),
                    ast::ComponentType::Mem => todo!(),
                    ast::ComponentType::Constraint => todo!(),
                }
            }
            ast::ComponentDef::Anon(component_type, component_body) => todo!(),
        };
        vec![node]
    }
}

fn elaborate_addrmap(name: String, body: &ast::ComponentBody, properties: Vec<&HashMap<String, PrimaryLiteral>) -> Node {
    let mut addrmap = AddrMap {
        name,
        properties: HashMap::new(),
        default_properties: HashMap::new(),
    };
    let mut children = Vec::new();

    for elem in &body.elements {
        match elem {
            ast::ComponentBodyElem::ComponentDef(component) => {
                let mut child = elaborate_component(component);
                children.append(&mut child);
            }
            ast::ComponentBodyElem::EnumDef(enum_def) => todo!(),
            ast::ComponentBodyElem::StructDef(struct_def) => todo!(),
            ast::ComponentBodyElem::ConstraintDef(constraint_def) => todo!(),
            ast::ComponentBodyElem::ExplicitComponentInst(explicit_component_inst) => todo!(),
            ast::ComponentBodyElem::PropertyAssignment(property_assignment) => {
                match property_assignment {
                    ast::PropertyAssignment::ExplicitOrDefaultPropAssignment(
                        ast::ExplicitOrDefaultPropAssignment::ExplicitPropModifier(
                            default_keyword,
                            explicit_prop_modifier,
                        ),
                    ) => todo!("property modifiers"),
                    ast::PropertyAssignment::ExplicitOrDefaultPropAssignment(
                        ast::ExplicitOrDefaultPropAssignment::ExplicitPropAssignment(
                            default_keyword,
                            explicit_property_assignment,
                        ),
                    ) => match explicit_property_assignment {
                        ast::ExplicitPropertyAssignment::Assignment(
                            identity_or_prop_keyword,
                            prop_assignment_rhs,
                        ) => {
                            let property_namespace = if default_keyword.is_some() {
                                &mut addrmap.default_properties
                            } else {
                                &mut addrmap.properties
                            };

                            let prop_id = match identity_or_prop_keyword {
                                ast::IdentityOrPropKeyword::Id(prop_id) => prop_id.clone(),
                                ast::IdentityOrPropKeyword::PropKeyword(prop_keyword) => {
                                    // treat as a string
                                    format!("{prop_keyword:?}").to_lowercase()
                                }
                            };

                            match prop_assignment_rhs {
                                Some(ast::PropAssignmentRhs::ConstantExpr(constant_expr)) => {
                                    let value = evaluate_constants(constant_expr);
                                    if property_namespace.contains_key(&prop_id) {
                                        panic!("duplicate property {prop_id}");
                                    }
                                    property_namespace.insert(prop_id, value);
                                }
                                Some(ast::PropAssignmentRhs::PrecedenceType(precedence_type)) => {
                                    todo!()
                                }
                                None => todo!(),
                            }
                        }
                        ast::ExplicitPropertyAssignment::EncodeAssignment(_) => {
                            todo!("property encode assignment")
                        }
                    },
                    ast::PropertyAssignment::PostPropAssignment(
                        ast::PostPropAssignment::PropRef(prop_ref, prop_assignment_rhs),
                    ) => {
                        todo!("property references")
                    }
                    ast::PropertyAssignment::PostPropAssignment(
                        ast::PostPropAssignment::PostEncodeAssignment(post_encode_assignment),
                    ) => todo!("encode properties"),
                }
            }
        }
    }

    Node {
        content: NodeContent::Addrmap(addrmap),
        children,
    }
}

fn elaborate_reg(name: String, body: &ast::ComponentBody) -> Node {
    // TODO: Do we need to distinguish between these?
    elaborate_addrmap(name, body)
}

fn elaborate_field(name: String, body: &ast::ComponentBody) -> Node {
    // TODO: Do we need to distinguish between these?
    elaborate_addrmap(name, body)
}

/// Evaluate a constant, resolving it down to a single value.
fn evaluate_constants(constexpr: &ast::ConstantExpr) -> ast::PrimaryLiteral {
    match constexpr {
        ast::ConstantExpr::ConstantPrimary(constant_primary, constant_expr_continue) => {
            match constant_primary {
                ast::ConstantPrimary::Base(constant_primary_base) => match constant_primary_base {
                    ast::ConstantPrimaryBase::PrimaryLiteral(primary_literal) => {
                        match constant_expr_continue {
                            Some(cont) => match &**cont {
                                ast::ConstantExprContinue::BinaryOp(
                                    binary_op,
                                    constant_expr,
                                    constant_expr_continue,
                                ) => {
                                    let lhs = match primary_literal {
                                        PrimaryLiteral::Number(v) => *v,
                                        _ => panic!(
                                            "unexpected type {primary_literal:?} on lhs of {binary_op:?}"
                                        ),
                                    };
                                    let rhs = evaluate_constants(constant_expr);
                                    let rhs = match rhs {
                                        PrimaryLiteral::Number(v) => v,
                                        _ => panic!(
                                            "unexpected type {rhs:?} on rhs of {binary_op:?}"
                                        ),
                                    };

                                    let result = match binary_op {
                                        ast::BinaryOp::LeftShift => lhs << rhs,
                                        binary_op => todo!("constexpr binary op {binary_op:?}"),
                                    };
                                    PrimaryLiteral::Number(result)
                                }
                                ast::ConstantExprContinue::TernaryOp(
                                    constant_expr,
                                    constant_expr1,
                                    constant_expr_continue,
                                ) => todo!(),
                            },
                            None => primary_literal.clone(),
                        }
                    }
                    ast::ConstantPrimaryBase::ConstantConcat(constant_exprs) => todo!(),
                    ast::ConstantPrimaryBase::ConstantMultipleConcat(
                        constant_expr,
                        constant_exprs,
                    ) => todo!(),
                    ast::ConstantPrimaryBase::ConstantExpr(constant_expr) => todo!(),
                    ast::ConstantPrimaryBase::SimpleTypeCast(integer_type, constant_expr) => {
                        todo!()
                    }
                    ast::ConstantPrimaryBase::BooleanCast(constant_expr) => todo!(),
                    ast::ConstantPrimaryBase::InstanceOrPropRef(instance_or_prop_ref) => todo!(),
                    ast::ConstantPrimaryBase::StructLiteral(_, struct_literal_elements) => todo!(),
                    ast::ConstantPrimaryBase::ArrayLiteral(constant_exprs) => todo!(),
                },
                ast::ConstantPrimary::Cast(constant_primary_base, constant_expr) => todo!(),
            }
        }
        ast::ConstantExpr::UnaryOp(unary_op, constant_expr, constant_expr_continue) => todo!(),
    }
}

#[derive(Debug, Clone)]
pub struct AddrMap {
    name: String,
    /// local properties, should not be propogated
    properties: HashMap<String, PrimaryLiteral>,
    /// properties set at this level as default, should be propogated down
    default_properties: HashMap<String, PrimaryLiteral>,
}

#[derive(Debug, Clone)]
pub struct Register {
    name: String,
    /// local properties, should not be propogated
    properties: HashMap<String, PrimaryLiteral>,
    /// properties set at this level as default, should be propogated down
    default_properties: HashMap<String, PrimaryLiteral>,
}

#[derive(Debug, Clone)]
pub struct Field {
    name: String,
    /// local properties, should not be propogated
    properties: HashMap<String, PrimaryLiteral>,
    /// properties set at this level as default, should be propogated down
    default_properties: HashMap<String, PrimaryLiteral>,
}

#[derive(Debug, Clone)]
pub struct Node {
    content: NodeContent,
    children: Vec<Node>,
}

#[derive(Debug, Clone)]
pub enum NodeContent {
    Addrmap(AddrMap),
    Register(Register),
    Field(Field),
}
