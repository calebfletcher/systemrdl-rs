// Generates VHDL code from a SystemRDL AST
// Usage: cargo run --example vhdlgen <file.rdl>

use anyhow::{Context, ensure};
use std::path::Path;

fn main() -> Result<(), anyhow::Error> {
    let arg = std::env::args_os()
        .nth(1)
        .context("no path to an rdl file was provided")?;
    let path = Path::new(&arg);

    ensure!(path.exists(), "path exists");
    let extension = path.extension().context("path has an extension")?;
    ensure!(extension == "rdl", "path points to an rdl file");

    let contents = std::fs::read_to_string(path).context("could not read rdl file")?;
    let ast = systemrdl::parse(&contents)?;

    let vhdl = vhdl_from_ast(&ast);
    println!("{}", vhdl);
    Ok(())
}

use systemrdl::ast::*;
use systemrdl::ast::{
    ConstantExpr, ConstantPrimary, ConstantPrimaryBase, ExplicitOrDefaultPropAssignment,
    ExplicitPropertyAssignment, PrimaryLiteral, PropAssignmentRhs,
};

fn vhdl_from_ast(ast: &Root) -> String {
    let mut vhdl = String::new();
    vhdl.push_str("-- Auto-generated VHDL from SystemRDL\n");
    for desc in &ast.descriptions {
        if let Description::ComponentDef(component) = desc {
            vhdl.push_str(&vhdl_component(component));
        }
    }
    vhdl
}

fn vhdl_component(component: &Component) -> String {
    let mut vhdl = String::new();
    match &component.def {
        ComponentDef::Named(ctype, name, _, body) => {
            vhdl.push_str(&format!("\nentity {} is\n", name));
            vhdl.push_str(&vhdl_ports(body));
            vhdl.push_str("end entity;\n\n");
            vhdl.push_str(&format!("architecture rtl of {} is\n", name));
            vhdl.push_str("begin\n");
            vhdl.push_str(&vhdl_signals(body));
            vhdl.push_str("end architecture;\n");
        }
        _ => {}
    }
    vhdl
}

fn vhdl_ports(body: &ComponentBody) -> String {
    let mut port_lines = Vec::new();
    // Add software bus interface ports (not registered)
    port_lines.push("        bus_addr : in std_logic_vector(31 downto 0)".to_string());
    port_lines.push("        bus_data_in : in std_logic_vector(31 downto 0)".to_string());
    port_lines.push("        bus_data_out : out std_logic_vector(31 downto 0)".to_string());
    port_lines.push("        bus_read_valid : in std_logic".to_string());
    port_lines.push("        bus_write_valid : in std_logic".to_string());
    // Find global/addrmap-level defaults
    let global_sw = find_default_property(body, "sw");
    let global_hw = find_default_property(body, "hw");
    for elem in &body.elements {
        if let ComponentBodyElem::ComponentDef(comp) = elem {
            let (reg_body, _reg_insts) = match &comp.def {
                ComponentDef::Named(ComponentType::Reg, _reg_name, _, reg_body) => (reg_body, comp.insts.as_ref()),
                ComponentDef::Anon(ComponentType::Reg, reg_body) => (reg_body, comp.insts.as_ref()),
                _ => continue,
            };
            let reg_sw = find_default_property(reg_body, "sw").or(global_sw);
            let reg_hw = find_default_property(reg_body, "hw").or(global_hw);
            for field_elem in &reg_body.elements {
                if let ComponentBodyElem::ComponentDef(field_comp) = field_elem {
                    let (field_name, field_body, _field_insts) = match &field_comp.def {
                        ComponentDef::Named(ComponentType::Field, field_name, _, field_body) => (field_name, field_body, field_comp.insts.as_ref()),
                        ComponentDef::Anon(ComponentType::Field, field_body) => {
                            let mut name = None;
                            let mut width = None;
                            if let Some(insts) = &field_comp.insts {
                                for inst in &insts.component_insts {
                                    name = Some(inst.id.clone());
                                    if let Some(ArrayOrRange::Array(arr)) = &inst.array_or_range {
                                        if let Some(ConstantExpr::ConstantPrimary(ConstantPrimary::Base(ConstantPrimaryBase::PrimaryLiteral(PrimaryLiteral::Number(w))), _)) = arr.get(0) {
                                            width = Some(*w as usize);
                                        }
                                    }
                                }
                            }
                            let fname = name.unwrap_or_else(|| "field".to_string());
                            let w = width.unwrap_or(1);
                            let (hw_access, _sw_access) = resolve_effective_access(field_body, reg_hw, reg_sw, global_hw, global_sw);
                            let hw_read = matches!(hw_access, AccessType::R | AccessType::Rw | AccessType::Rw1);
                            let hw_write = matches!(hw_access, AccessType::W | AccessType::W1 | AccessType::Rw | AccessType::Rw1);
                            if hw_read && hw_write {
                                let vhdl_type = if w > 1 {
                                    format!("std_logic_vector({} downto 0)", w - 1)
                                } else {
                                    "std_logic".to_string()
                                };
                                port_lines.push(format!("        {}_out : out {}", fname, vhdl_type));
                                port_lines.push(format!("        {}_in : in {}", fname, vhdl_type));
                                port_lines.push(format!("        {}_we : in std_logic", fname));
                            } else if hw_read {
                                let vhdl_type = if w > 1 {
                                    format!("std_logic_vector({} downto 0)", w - 1)
                                } else {
                                    "std_logic".to_string()
                                };
                                port_lines.push(format!("        {} : out {}", fname, vhdl_type));
                            } else if hw_write {
                                let vhdl_type = if w > 1 {
                                    format!("std_logic_vector({} downto 0)", w - 1)
                                } else {
                                    "std_logic".to_string()
                                };
                                port_lines.push(format!("        {} : in {}", fname, vhdl_type));
                                port_lines.push(format!("        {}_we : in std_logic", fname));
                            }
                            continue;
                        }
                        _ => continue,
                    };
                    let (base_name, width) = parse_field_width(field_name);
                    let (hw_access, _sw_access) = resolve_effective_access(field_body, reg_hw, reg_sw, global_hw, global_sw);
                    let hw_read = matches!(hw_access, AccessType::R | AccessType::Rw | AccessType::Rw1);
                    let hw_write = matches!(hw_access, AccessType::W | AccessType::W1 | AccessType::Rw | AccessType::Rw1);
                    if hw_read && hw_write {
                        let vhdl_type = if width > 1 {
                            format!("std_logic_vector({} downto 0)", width - 1)
                        } else {
                            "std_logic".to_string()
                        };
                        port_lines.push(format!("        {}_out : out {}", base_name, vhdl_type));
                        port_lines.push(format!("        {}_in : in {}", base_name, vhdl_type));
                        port_lines.push(format!("        {}_we : in std_logic", base_name));
                    } else if hw_read {
                        let vhdl_type = if width > 1 {
                            format!("std_logic_vector({} downto 0)", width - 1)
                        } else {
                            "std_logic".to_string()
                        };
                        port_lines.push(format!("        {} : out {}", base_name, vhdl_type));
                    } else if hw_write {
                        let vhdl_type = if width > 1 {
                            format!("std_logic_vector({} downto 0)", width - 1)
                        } else {
                            "std_logic".to_string()
                        };
                        port_lines.push(format!("        {} : in {}", base_name, vhdl_type));
                        port_lines.push(format!("        {}_we : in std_logic", base_name));
                    }
                }
            }
        }
    }
    if port_lines.is_empty() {
        String::from("")
    } else {
        let mut ports = String::from("    port (\n");
        ports.push_str(&port_lines.join(";\n"));
        ports.push_str("\n    );\n");
        ports
    }
}

// Find the default property ("sw" or "hw") in a component body
fn find_default_property(body: &ComponentBody, prop: &str) -> Option<AccessType> {
    for elem in &body.elements {
        if let ComponentBodyElem::PropertyAssignment(pa) = elem {
            if let Some(a) = extract_access_type_for(pa, prop) {
                return Some(a);
            }
        }
    }
    None
}

// Extract access type for a specific property ("sw" or "hw")
fn extract_access_type_for(pa: &PropertyAssignment, prop: &str) -> Option<AccessType> {
    match pa {
        PropertyAssignment::ExplicitOrDefaultPropAssignment(e) => match e {
            ExplicitOrDefaultPropAssignment::ExplicitPropAssignment(_, ex) => {
                match ex {
                    ExplicitPropertyAssignment::Assignment(id, Some(PropAssignmentRhs::ConstantExpr(expr))) => {
                        let matches_prop = match id {
                            IdentityOrPropKeyword::Id(s) => s == prop,
                            IdentityOrPropKeyword::PropKeyword(k) => k.to_string() == prop,
                        };
                        if matches_prop {
                            if let ConstantExpr::ConstantPrimary(
                                ConstantPrimary::Base(ConstantPrimaryBase::PrimaryLiteral(PrimaryLiteral::AccessTypeLiteral(a))),
                                _,
                            ) = expr {
                                Some(*a)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            ExplicitOrDefaultPropAssignment::ExplicitPropModifier(_, exmod) => {
                if exmod.id == prop {
                    // No access type in modifier, so skip
                    None
                } else {
                    None
                }
            }
        },
        _ => None,
    }
}

// Resolve effective access type for a field, cascading through field, reg, and global defaults
fn resolve_effective_access(
    field_body: &ComponentBody,
    reg_hw: Option<AccessType>,
    reg_sw: Option<AccessType>,
    global_hw: Option<AccessType>,
    global_sw: Option<AccessType>,
) -> (AccessType, AccessType) {
    // Field-level overrides
    let field_hw = find_default_property(field_body, "hw");
    let field_sw = find_default_property(field_body, "sw");
    let hw = field_hw.or(reg_hw).or(global_hw).unwrap_or(AccessType::Rw);
    let sw = field_sw.or(reg_sw).or(global_sw).unwrap_or(AccessType::Rw);
    (hw, sw)
}

fn extract_field_info(body: &ComponentBody) -> (usize, Option<AccessType>) {
    // Only extract access type from field body property assignments
    let mut access = None;
    for elem in &body.elements {
        if let ComponentBodyElem::PropertyAssignment(pa) = elem {
            if let Some(a) = extract_access_type(pa) {
                access = Some(a);
            }
        }
    }
    (1, access)
}

fn find_default_access(body: &ComponentBody) -> Option<AccessType> {
    for elem in &body.elements {
        if let ComponentBodyElem::PropertyAssignment(pa) = elem {
            if let Some(a) = extract_access_type(pa) {
                return Some(a);
            }
        }
    }
    None
}

fn extract_access_type(pa: &PropertyAssignment) -> Option<AccessType> {
    match pa {
        PropertyAssignment::ExplicitOrDefaultPropAssignment(e) => match e {
            ExplicitOrDefaultPropAssignment::ExplicitPropAssignment(_, ex) => match ex {
                ExplicitPropertyAssignment::Assignment(
                    _,
                    Some(PropAssignmentRhs::ConstantExpr(expr)),
                ) => {
                    if let ConstantExpr::ConstantPrimary(
                        ConstantPrimary::Base(ConstantPrimaryBase::PrimaryLiteral(
                            PrimaryLiteral::AccessTypeLiteral(a),
                        )),
                        _,
                    ) = expr
                    {
                        Some(*a)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn parse_field_width(field_name: &str) -> (String, usize) {
    // Try to extract width from field name like foo[7:0]
    if let Some(idx) = field_name.find('[') {
        let base = &field_name[..idx];
        let rest = &field_name[idx + 1..field_name.len() - 1];
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() == 2 {
            let msb: usize = parts[0].parse().unwrap_or(0);
            let lsb: usize = parts[1].parse().unwrap_or(0);
            let width = if msb >= lsb { msb - lsb + 1 } else { 1 };
            return (base.to_string(), width);
        }
    }
    (field_name.to_string(), 1)
}

fn vhdl_signals(body: &ComponentBody) -> String {
    let mut sigs = String::new();
    for elem in &body.elements {
        if let ComponentBodyElem::ComponentDef(comp) = elem {
            if let ComponentDef::Named(ComponentType::Reg, _reg_name, _, reg_body) = &comp.def {
                for field_elem in &reg_body.elements {
                    if let ComponentBodyElem::ComponentDef(field_comp) = field_elem {
                        if let ComponentDef::Named(ComponentType::Field, field_name, _, _) =
                            &field_comp.def
                        {
                            // TODO: Emit signal and logic for HW/SW attributes
                            sigs.push_str(&format!("    -- HW/SW logic for {} here\n", field_name));
                        }
                    }
                }
            }
        }
    }
    sigs
}
