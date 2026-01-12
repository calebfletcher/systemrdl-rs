#![allow(unused)]

use std::{collections::HashMap, hash::Hash};

use crate::ast::{self, PrimaryLiteral};

#[derive(Debug, Clone)]
pub struct RootNamespace {
    addrmaps: Vec<AddrMap>,
}

pub fn elaborate(ast: ast::Root) -> Result<RootNamespace, anyhow::Error> {
    let mut root = RootNamespace {
        addrmaps: Vec::new(),
    };

    for desc in ast.descriptions {
        match desc {
            ast::Description::ComponentDef(component) => {
                assert!(component.inst_type.is_none());
                assert!(component.insts.is_none());
                match component.def {
                    ast::ComponentDef::Named(component_type, name, param_def, component_body) => {
                        match component_type {
                            ast::ComponentType::Field => todo!(),
                            ast::ComponentType::Reg => todo!(),
                            ast::ComponentType::RegFile => todo!(),
                            ast::ComponentType::AddrMap => {
                                assert!(param_def.is_none());
                                root.addrmaps.push(elaborate_addrmap(name, component_body));
                            }
                            ast::ComponentType::Signal => todo!(),
                            ast::ComponentType::Enum => todo!(),
                            ast::ComponentType::EnumVariant => todo!(),
                            ast::ComponentType::Mem => todo!(),
                            ast::ComponentType::Constraint => todo!(),
                        }
                    }
                    ast::ComponentDef::Anon(component_type, component_body) => todo!(),
                }
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

fn elaborate_addrmap(name: String, body: ast::ComponentBody) -> AddrMap {
    let mut addrmap = AddrMap {
        name,
        properties: HashMap::new(),
        default_properties: HashMap::new(),
    };

    for elem in body.elements {
        match elem {
            ast::ComponentBodyElem::ComponentDef(component) => {
                continue;
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
                                ast::IdentityOrPropKeyword::Id(prop_id) => prop_id,
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

    addrmap
}

/// Evaluate a
fn evaluate_constants(constexpr: ast::ConstantExpr) -> ast::PrimaryLiteral {
    match constexpr {
        ast::ConstantExpr::ConstantPrimary(constant_primary, constant_expr_continue) => {
            assert!(constant_expr_continue.is_none());
            match constant_primary {
                ast::ConstantPrimary::Base(constant_primary_base) => match constant_primary_base {
                    ast::ConstantPrimaryBase::PrimaryLiteral(primary_literal) => primary_literal,
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
