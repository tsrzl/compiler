//! Name binding for declarations represented in the parsed syntax trees.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::enum_values::{EnumValue, values_for_enum};
use crate::module_resolver;
use crate::syntax::{
    ClassDeclaration, ClassMember, EnumDeclaration, Expression, FunctionDeclaration,
    FunctionParameter, Statement, SyntaxTree, TypeReference,
};

#[derive(Debug, Default)]
pub(crate) struct SymbolTable {
    type_aliases: HashMap<PathBuf, HashMap<String, Vec<String>>>,
    interfaces: HashMap<PathBuf, HashMap<String, Vec<InterfacePropertyType>>>,
    classes: HashMap<PathBuf, HashSet<String>>,
    enums: HashMap<PathBuf, HashSet<String>>,
    enum_member_primitives: HashMap<PathBuf, HashMap<String, HashMap<String, &'static str>>>,
    class_properties: HashMap<PathBuf, HashMap<String, Vec<InterfacePropertyType>>>,
    class_static_properties: HashMap<PathBuf, HashMap<String, Vec<InterfacePropertyType>>>,
    class_bases: HashMap<PathBuf, HashMap<String, String>>,
    class_constructors: HashMap<PathBuf, HashMap<String, Vec<FunctionParameter>>>,
    module_type_exports: HashMap<PathBuf, HashMap<String, TypeTarget>>,
    imported_types: HashMap<PathBuf, HashMap<String, TypeTarget>>,
    imported_values: HashMap<PathBuf, HashMap<String, Vec<String>>>,
    module_exports: HashMap<PathBuf, HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypeTarget {
    module_path: PathBuf,
    name: String,
}

pub(crate) struct ScopedSymbolTable<'symbols> {
    symbols: &'symbols SymbolTable,
    module_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct InterfacePropertyType {
    pub(crate) name: String,
    pub(crate) type_names: Vec<String>,
    pub(crate) optional: bool,
    pub(crate) modifiers: PropertyModifiers,
    pub(crate) declaring_class: Option<String>,
    pub(crate) method_return_types: Option<Vec<String>>,
    pub(crate) method_parameters: Option<Vec<FunctionParameter>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PropertyAccessibility {
    Private,
    Protected,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PropertyModifiers {
    pub(crate) accessibility: Option<PropertyAccessibility>,
    pub(crate) readonly: bool,
}

impl SymbolTable {
    pub(crate) fn for_source(&self, source_path: &Path) -> ScopedSymbolTable<'_> {
        ScopedSymbolTable {
            symbols: self,
            module_path: module_resolver::normalize_path(source_path),
        }
    }

    fn resolve_type_target(&self, module_path: &Path, name: &str) -> Option<TypeTarget> {
        let module_path = module_resolver::normalize_path(module_path);
        if self
            .type_aliases
            .get(&module_path)
            .is_some_and(|aliases| aliases.contains_key(name))
            || self
                .interfaces
                .get(&module_path)
                .is_some_and(|interfaces| interfaces.contains_key(name))
            || self
                .classes
                .get(&module_path)
                .is_some_and(|classes| classes.contains(name))
            || self
                .enums
                .get(&module_path)
                .is_some_and(|enums| enums.contains(name))
        {
            return Some(TypeTarget {
                module_path,
                name: name.to_owned(),
            });
        }
        let resolved_import = self
            .imported_types
            .get(&module_path)
            .and_then(|imports| imports.get(name))
            .or_else(|| {
                self.module_type_exports
                    .get(&module_path)
                    .and_then(|exports| exports.get(name))
            })
            .cloned();
        if resolved_import.is_some() || module_path.as_os_str().is_empty() {
            return resolved_import;
        }
        let global_scope = Path::new("");
        if self
            .type_aliases
            .get(global_scope)
            .is_some_and(|aliases| aliases.contains_key(name))
            || self
                .interfaces
                .get(global_scope)
                .is_some_and(|interfaces| interfaces.contains_key(name))
            || self
                .classes
                .get(global_scope)
                .is_some_and(|classes| classes.contains(name))
            || self
                .enums
                .get(global_scope)
                .is_some_and(|enums| enums.contains(name))
        {
            return Some(TypeTarget {
                module_path: PathBuf::new(),
                name: name.to_owned(),
            });
        }
        None
    }

    fn is_same_or_derived_from(
        &self,
        candidate: &TypeTarget,
        ancestor: &TypeTarget,
        visited: &mut HashSet<(PathBuf, String)>,
    ) -> bool {
        if candidate == ancestor {
            return true;
        }
        if !visited.insert((candidate.module_path.clone(), candidate.name.clone())) {
            return false;
        }
        if let Some(aliases) = self
            .type_aliases
            .get(&candidate.module_path)
            .and_then(|aliases| aliases.get(&candidate.name))
            && aliases.len() == 1
            && let Some(alias) = self.resolve_type_target(&candidate.module_path, &aliases[0])
            && self.is_same_or_derived_from(&alias, ancestor, visited)
        {
            return true;
        }
        let Some(base_class) = self
            .class_bases
            .get(&candidate.module_path)
            .and_then(|classes| classes.get(&candidate.name))
        else {
            return false;
        };
        let Some(base_target) = self.resolve_type_target(&candidate.module_path, base_class) else {
            return false;
        };
        self.is_same_or_derived_from(&base_target, ancestor, visited)
    }
}

impl ScopedSymbolTable<'_> {
    pub(crate) fn is_same_or_derived_from(
        &self,
        candidate_name: &str,
        ancestor_name: &str,
    ) -> bool {
        let Some(candidate) = self
            .symbols
            .resolve_type_target(&self.module_path, candidate_name)
        else {
            return false;
        };
        let Some(ancestor) = self
            .symbols
            .resolve_type_target(&self.module_path, ancestor_name)
        else {
            return false;
        };
        self.symbols
            .is_same_or_derived_from(&candidate, &ancestor, &mut HashSet::new())
    }

    pub(crate) fn is_type_name_defined(&self, name: &str) -> bool {
        name.strip_suffix("[]")
            .is_some_and(|element_type| self.is_type_name_defined(element_type))
            || is_builtin_type(name)
            || self
                .symbols
                .resolve_type_target(&self.module_path, name)
                .is_some()
    }

    pub(crate) fn resolve_annotation(&self, annotation: &TypeReference) -> Vec<String> {
        if annotation.predicate_parameter().is_some() {
            return vec!["boolean".to_owned()];
        }
        let type_spellings = annotation.type_spellings().collect::<Vec<_>>();
        self.resolve_names(&type_spellings)
    }

    pub(crate) fn resolve_names(&self, names: &[String]) -> Vec<String> {
        let mut resolved = Vec::new();
        for name in names {
            let mut visited = HashSet::new();
            for type_name in self.resolve_type_name(&self.module_path, name, &mut visited) {
                if !resolved.contains(&type_name) {
                    resolved.push(type_name);
                }
            }
        }
        resolved
    }

    pub(crate) fn interface_properties(&self, name: &str) -> Option<Vec<InterfacePropertyType>> {
        self.find_interface_properties(&self.module_path, name, &mut HashSet::new())
    }

    pub(crate) fn properties_for_type(&self, name: &str) -> Option<Vec<InterfacePropertyType>> {
        if let Some(class_name) = name.strip_prefix("typeof ") {
            self.find_class_static_properties(&self.module_path, class_name, &mut HashSet::new())
        } else {
            self.interface_properties(name)
        }
    }

    pub(crate) fn constructor_parameters(&self, name: &str) -> Option<Vec<FunctionParameter>> {
        self.find_constructor_parameters(&self.module_path, name, &mut HashSet::new())
    }

    pub(crate) fn imported_values(&self) -> Option<&HashMap<String, Vec<String>>> {
        self.symbols.imported_values.get(&self.module_path)
    }

    pub(crate) fn is_enum_type(&self, name: &str) -> bool {
        self.symbols
            .resolve_type_target(&self.module_path, name)
            .is_some_and(|target| {
                self.symbols
                    .enums
                    .get(&target.module_path)
                    .is_some_and(|enums| enums.contains(&target.name))
            })
    }

    pub(crate) fn are_same_enum_type(&self, left: &str, right: &str) -> bool {
        self.symbols
            .resolve_type_target(&self.module_path, left)
            .zip(self.symbols.resolve_type_target(&self.module_path, right))
            .is_some_and(|(left, right)| left == right)
    }

    pub(crate) fn enum_member_underlying_type(
        &self,
        enum_name: &str,
        member_name: &str,
    ) -> Option<&'static str> {
        let target = self
            .symbols
            .resolve_type_target(&self.module_path, enum_name)?;
        self.symbols
            .enum_member_primitives
            .get(&target.module_path)?
            .get(&target.name)?
            .get(member_name)
            .copied()
    }

    pub(crate) fn enum_underlying_type(&self, name: &str) -> Option<&'static str> {
        let target = self.symbols.resolve_type_target(&self.module_path, name)?;
        let members = self
            .symbols
            .enum_member_primitives
            .get(&target.module_path)?
            .get(&target.name)?;
        let mut primitive = None;
        for member_primitive in members.values() {
            if *member_primitive == "unknown" {
                return None;
            }
            if primitive.is_some_and(|primitive| primitive != *member_primitive) {
                return None;
            }
            primitive = Some(*member_primitive);
        }
        primitive
    }

    fn resolve_type_name(
        &self,
        module_path: &Path,
        name: &str,
        visited: &mut HashSet<(PathBuf, String)>,
    ) -> Vec<String> {
        let Some(target) = self.symbols.resolve_type_target(module_path, name) else {
            return vec![name.to_owned()];
        };
        if self.is_interface_type(&target, &mut HashSet::new()) {
            return vec![name.to_owned()];
        }
        let key = (target.module_path.clone(), target.name.clone());
        if !visited.insert(key) {
            return vec![name.to_owned()];
        }
        let Some(aliases) = self
            .symbols
            .type_aliases
            .get(&target.module_path)
            .and_then(|aliases| aliases.get(&target.name))
        else {
            return vec![name.to_owned()];
        };
        aliases
            .iter()
            .flat_map(|alias| self.resolve_type_name(&target.module_path, alias, visited))
            .collect()
    }

    fn is_interface_type(
        &self,
        target: &TypeTarget,
        visited: &mut HashSet<(PathBuf, String)>,
    ) -> bool {
        let key = (target.module_path.clone(), target.name.clone());
        if !visited.insert(key) {
            return false;
        }
        if self
            .symbols
            .interfaces
            .get(&target.module_path)
            .is_some_and(|interfaces| interfaces.contains_key(&target.name))
        {
            return true;
        }
        let Some(aliases) = self
            .symbols
            .type_aliases
            .get(&target.module_path)
            .and_then(|aliases| aliases.get(&target.name))
        else {
            return false;
        };
        if aliases.len() != 1 {
            return false;
        }
        self.symbols
            .resolve_type_target(&target.module_path, &aliases[0])
            .is_some_and(|target| self.is_interface_type(&target, visited))
    }

    fn find_interface_properties(
        &self,
        module_path: &Path,
        name: &str,
        visited: &mut HashSet<(PathBuf, String)>,
    ) -> Option<Vec<InterfacePropertyType>> {
        let target = self.symbols.resolve_type_target(module_path, name)?;
        let key = (target.module_path.clone(), target.name.clone());
        if !visited.insert(key) {
            return None;
        }
        if let Some(properties) = self
            .symbols
            .interfaces
            .get(&target.module_path)
            .and_then(|interfaces| interfaces.get(&target.name))
        {
            return Some(properties.clone());
        }
        if let Some(properties) = self
            .symbols
            .class_properties
            .get(&target.module_path)
            .and_then(|classes| classes.get(&target.name))
        {
            let mut inherited = self
                .symbols
                .class_bases
                .get(&target.module_path)
                .and_then(|classes| classes.get(&target.name))
                .and_then(|base_class| {
                    self.find_interface_properties(&target.module_path, base_class, visited)
                })
                .unwrap_or_default();
            for property in properties {
                if let Some(inherited_property) = inherited
                    .iter_mut()
                    .find(|inherited_property| inherited_property.name == property.name)
                {
                    *inherited_property = property.clone();
                } else {
                    inherited.push(property.clone());
                }
            }
            return Some(inherited);
        }
        let aliases = self
            .symbols
            .type_aliases
            .get(&target.module_path)
            .and_then(|aliases| aliases.get(&target.name))?;
        if aliases.len() != 1 {
            return None;
        }
        self.find_interface_properties(&target.module_path, &aliases[0], visited)
    }

    fn find_class_static_properties(
        &self,
        module_path: &Path,
        name: &str,
        visited: &mut HashSet<(PathBuf, String)>,
    ) -> Option<Vec<InterfacePropertyType>> {
        let target = self.symbols.resolve_type_target(module_path, name)?;
        let key = (target.module_path.clone(), target.name.clone());
        if !visited.insert(key) {
            return None;
        }
        let properties = self
            .symbols
            .class_static_properties
            .get(&target.module_path)
            .and_then(|classes| classes.get(&target.name));
        let base_properties = self
            .symbols
            .class_bases
            .get(&target.module_path)
            .and_then(|classes| classes.get(&target.name))
            .and_then(|base_class| {
                self.find_class_static_properties(&target.module_path, base_class, visited)
            });
        let mut inherited = base_properties.unwrap_or_default();
        if let Some(properties) = properties {
            for property in properties {
                if let Some(inherited_property) = inherited
                    .iter_mut()
                    .find(|inherited_property| inherited_property.name == property.name)
                {
                    *inherited_property = property.clone();
                } else {
                    inherited.push(property.clone());
                }
            }
            return Some(inherited);
        }
        (!inherited.is_empty()).then_some(inherited)
    }

    fn find_constructor_parameters(
        &self,
        module_path: &Path,
        name: &str,
        visited: &mut HashSet<(PathBuf, String)>,
    ) -> Option<Vec<FunctionParameter>> {
        let target = self.symbols.resolve_type_target(module_path, name)?;
        let key = (target.module_path.clone(), target.name.clone());
        if !visited.insert(key) {
            return None;
        }
        if let Some(parameters) = self
            .symbols
            .class_constructors
            .get(&target.module_path)
            .and_then(|classes| classes.get(&target.name))
        {
            return Some(parameters.clone());
        }
        let aliases = self
            .symbols
            .type_aliases
            .get(&target.module_path)
            .and_then(|aliases| aliases.get(&target.name))?;
        if aliases.len() != 1 {
            return None;
        }
        self.find_constructor_parameters(&target.module_path, &aliases[0], visited)
    }
}

fn is_builtin_type(name: &str) -> bool {
    matches!(
        name,
        "any"
            | "unknown"
            | "never"
            | "void"
            | "undefined"
            | "null"
            | "boolean"
            | "number"
            | "string"
            | "symbol"
            | "bigint"
            | "object"
            | "Array"
            | "ReadonlyArray"
            | "Object"
            | "Function"
            | "String"
            | "Number"
            | "Boolean"
            | "Symbol"
            | "BigInt"
            | "Date"
            | "RegExp"
            | "Error"
            | "EvalError"
            | "RangeError"
            | "ReferenceError"
            | "SyntaxError"
            | "TypeError"
            | "URIError"
            | "Promise"
            | "PromiseLike"
            | "Map"
            | "ReadonlyMap"
            | "WeakMap"
            | "Set"
            | "ReadonlySet"
            | "WeakSet"
            | "Iterable"
            | "Iterator"
            | "AsyncIterable"
            | "AsyncIterator"
            | "Generator"
            | "AsyncGenerator"
            | "IArguments"
            | "PropertyKey"
            | "Partial"
            | "Required"
            | "Readonly"
            | "Pick"
            | "Omit"
            | "Exclude"
            | "Extract"
            | "NonNullable"
            | "Parameters"
            | "ConstructorParameters"
            | "ReturnType"
            | "InstanceType"
            | "ThisParameterType"
            | "OmitThisParameter"
            | "ThisType"
            | "Awaited"
            | "Uppercase"
            | "Lowercase"
            | "Capitalize"
            | "Uncapitalize"
    )
}

pub(crate) fn bind(syntax_trees: &[SyntaxTree]) -> SymbolTable {
    let mut symbols = SymbolTable::default();
    bind_declared_types(syntax_trees, &mut symbols);
    bind_declared_type_exports(syntax_trees, &mut symbols);
    bind_reexported_type_exports(syntax_trees, &mut symbols);
    bind_imported_types(syntax_trees, &mut symbols);

    let (local_values, declared_exports) = collect_local_values(syntax_trees, &symbols);
    symbols.module_exports = resolve_module_exports(&local_values, declared_exports);
    bind_reexported_values(syntax_trees, &mut symbols);
    bind_imported_values(syntax_trees, &mut symbols);
    symbols
}

fn bind_declared_types(syntax_trees: &[SyntaxTree], symbols: &mut SymbolTable) {
    for syntax_tree in syntax_trees {
        let module_path = source_scope_path(syntax_tree);
        for statement in syntax_tree.program().statements() {
            let declaration = statement.declaration();
            if let Statement::TypeAliasDeclaration(declaration) = declaration {
                symbols
                    .type_aliases
                    .entry(module_path.clone())
                    .or_default()
                    .insert(
                        declaration.name().to_owned(),
                        declaration.type_annotation().type_spellings().collect(),
                    );
            } else if let Statement::InterfaceDeclaration(declaration) = declaration {
                symbols
                    .interfaces
                    .entry(module_path.clone())
                    .or_default()
                    .insert(
                        declaration.name().to_owned(),
                        declaration
                            .members()
                            .iter()
                            .map(|property| InterfacePropertyType {
                                name: property.name().to_owned(),
                                type_names: property.type_annotation().type_spellings().collect(),
                                optional: property.is_optional(),
                                modifiers: PropertyModifiers::default(),
                                declaring_class: None,
                                method_return_types: None,
                                method_parameters: None,
                            })
                            .collect(),
                    );
            } else if let Statement::ClassDeclaration(declaration) = declaration {
                bind_class_declaration(declaration, &module_path, symbols);
            } else if let Statement::EnumDeclaration(declaration) = declaration {
                bind_enum_declaration(declaration, &module_path, symbols);
            }
        }
    }
}

fn bind_enum_declaration(
    declaration: &EnumDeclaration,
    module_path: &Path,
    symbols: &mut SymbolTable,
) {
    symbols
        .enums
        .entry(module_path.to_path_buf())
        .or_default()
        .insert(declaration.name().to_owned());
    let mut member_primitives = HashMap::new();
    let members = declaration
        .members()
        .iter()
        .zip(values_for_enum(declaration))
        .map(|(member, value)| {
            let (type_name, primitive) = match value {
                EnumValue::Number(_) => (
                    format!("{}.{}", declaration.name(), member.name()),
                    "number",
                ),
                EnumValue::String(_) => (
                    format!("{}.{}", declaration.name(), member.name()),
                    "string",
                ),
                EnumValue::Computed => (declaration.name().to_owned(), "unknown"),
            };
            member_primitives.insert(member.name().to_owned(), primitive);
            InterfacePropertyType {
                name: member.name().to_owned(),
                type_names: vec![type_name],
                optional: false,
                modifiers: PropertyModifiers::default(),
                declaring_class: None,
                method_return_types: None,
                method_parameters: None,
            }
        })
        .collect();
    symbols
        .enum_member_primitives
        .entry(module_path.to_path_buf())
        .or_default()
        .insert(declaration.name().to_owned(), member_primitives);
    symbols
        .class_static_properties
        .entry(module_path.to_path_buf())
        .or_default()
        .insert(declaration.name().to_owned(), members);
}

fn bind_class_declaration(
    declaration: &ClassDeclaration,
    module_path: &Path,
    symbols: &mut SymbolTable,
) {
    symbols
        .classes
        .entry(module_path.to_path_buf())
        .or_default()
        .insert(declaration.name().to_owned());
    let constructor_parameters = declaration
        .members()
        .iter()
        .find_map(|member| match member {
            ClassMember::Method(method) if method.name() == "constructor" => {
                Some(method.parameters().to_vec())
            }
            ClassMember::Method(_) | ClassMember::Property(_) => None,
        })
        .unwrap_or_default();
    symbols
        .class_constructors
        .entry(module_path.to_path_buf())
        .or_default()
        .insert(
            declaration.name().to_owned(),
            constructor_parameters.clone(),
        );
    if let Some(base_class) = declaration.base_class() {
        symbols
            .class_bases
            .entry(module_path.to_path_buf())
            .or_default()
            .insert(declaration.name().to_owned(), base_class.to_owned());
    }
    let mut class_properties = declaration
        .members()
        .iter()
        .filter_map(|member| match member {
            ClassMember::Method(method) if method.name() == "constructor" => None,
            _ if member.is_static() => None,
            _ => Some(class_member_type(member, declaration.name())),
        })
        .collect::<Vec<_>>();
    let class_static_properties = declaration
        .members()
        .iter()
        .filter(|member| member.is_static())
        .map(|member| class_member_type(member, declaration.name()))
        .collect();
    class_properties.extend(
        constructor_parameters
            .iter()
            .filter(|parameter| parameter.is_parameter_property())
            .map(|parameter| class_parameter_property_type(parameter, declaration.name())),
    );
    symbols
        .class_properties
        .entry(module_path.to_path_buf())
        .or_default()
        .insert(declaration.name().to_owned(), class_properties);
    symbols
        .class_static_properties
        .entry(module_path.to_path_buf())
        .or_default()
        .insert(declaration.name().to_owned(), class_static_properties);
}

fn class_parameter_property_type(
    parameter: &FunctionParameter,
    declaring_class: &str,
) -> InterfacePropertyType {
    InterfacePropertyType {
        name: parameter.name().to_owned(),
        type_names: parameter
            .type_annotation()
            .map(|annotation| annotation.type_spellings().collect())
            .or_else(|| parameter.initializer().map(infer_expression_types))
            .unwrap_or_else(|| vec!["any".to_owned()]),
        optional: parameter.is_optional() && parameter.initializer().is_none(),
        modifiers: PropertyModifiers {
            accessibility: if parameter.is_private_parameter_property() {
                Some(PropertyAccessibility::Private)
            } else if parameter.is_protected_parameter_property() {
                Some(PropertyAccessibility::Protected)
            } else {
                None
            },
            readonly: parameter.is_readonly_parameter_property(),
        },
        declaring_class: Some(declaring_class.to_owned()),
        method_return_types: None,
        method_parameters: None,
    }
}

fn class_member_type(member: &ClassMember, declaring_class: &str) -> InterfacePropertyType {
    match member {
        ClassMember::Property(property) => InterfacePropertyType {
            name: property.name().to_owned(),
            type_names: property
                .type_annotation()
                .map(|annotation| annotation.type_spellings().collect())
                .or_else(|| property.initializer().map(infer_expression_types))
                .unwrap_or_else(|| vec!["any".to_owned()]),
            optional: false,
            modifiers: PropertyModifiers {
                accessibility: if property.is_private() {
                    Some(PropertyAccessibility::Private)
                } else if property.is_protected() {
                    Some(PropertyAccessibility::Protected)
                } else {
                    None
                },
                readonly: property.is_readonly(),
            },
            declaring_class: Some(declaring_class.to_owned()),
            method_return_types: None,
            method_parameters: None,
        },
        ClassMember::Method(method) => InterfacePropertyType {
            name: method.name().to_owned(),
            type_names: vec!["any".to_owned()],
            optional: false,
            modifiers: PropertyModifiers {
                accessibility: if method.is_private() {
                    Some(PropertyAccessibility::Private)
                } else if method.is_protected() {
                    Some(PropertyAccessibility::Protected)
                } else {
                    None
                },
                readonly: false,
            },
            declaring_class: Some(declaring_class.to_owned()),
            method_return_types: Some(infer_class_method_return_types(method)),
            method_parameters: Some(method.parameters().to_vec()),
        },
    }
}

fn infer_class_method_return_types(method: &FunctionDeclaration) -> Vec<String> {
    if let Some(return_type) = method.return_type() {
        return return_type.type_spellings().collect();
    }

    let mut return_types = Vec::new();
    for expression in method.return_expressions() {
        let expression_types =
            expression.map_or_else(|| vec!["undefined".to_owned()], infer_expression_types);
        if expression_types.iter().any(|type_name| type_name == "any") {
            return vec!["any".to_owned()];
        }
        for type_name in expression_types {
            if !return_types.contains(&type_name) {
                return_types.push(type_name);
            }
        }
    }
    if return_types.is_empty() {
        vec!["void".to_owned()]
    } else {
        return_types
    }
}

fn source_scope_path(syntax_tree: &SyntaxTree) -> PathBuf {
    if syntax_tree
        .program()
        .statements()
        .iter()
        .any(Statement::is_external_module_indicator)
    {
        module_resolver::normalize_path(syntax_tree.source_file().path())
    } else {
        PathBuf::new()
    }
}

fn bind_declared_type_exports(syntax_trees: &[SyntaxTree], symbols: &mut SymbolTable) {
    for syntax_tree in syntax_trees {
        let module_path = module_resolver::normalize_path(syntax_tree.source_file().path());
        for statement in syntax_tree.program().statements() {
            if statement.is_exported()
                && let Some(type_name) = statement
                    .as_type_alias_declaration()
                    .map(crate::syntax::TypeAliasDeclaration::name)
                    .or_else(|| {
                        statement
                            .as_interface_declaration()
                            .map(crate::syntax::InterfaceDeclaration::name)
                    })
                    .or_else(|| {
                        statement
                            .as_class_declaration()
                            .map(crate::syntax::ClassDeclaration::name)
                    })
                    .or_else(|| {
                        statement
                            .as_enum_declaration()
                            .map(crate::syntax::EnumDeclaration::name)
                    })
            {
                symbols
                    .module_type_exports
                    .entry(module_path.clone())
                    .or_default()
                    .insert(
                        type_name.to_owned(),
                        TypeTarget {
                            module_path: module_path.clone(),
                            name: type_name.to_owned(),
                        },
                    );
            }
            if let Statement::ExportNamed(specifiers) | Statement::ExportTypeNamed(specifiers) =
                statement.declaration()
            {
                for specifier in specifiers {
                    let local_name = specifier.local_name();
                    if symbols
                        .type_aliases
                        .get(&module_path)
                        .is_some_and(|aliases| aliases.contains_key(local_name))
                        || symbols
                            .interfaces
                            .get(&module_path)
                            .is_some_and(|interfaces| interfaces.contains_key(local_name))
                        || symbols
                            .classes
                            .get(&module_path)
                            .is_some_and(|classes| classes.contains(local_name))
                        || symbols
                            .enums
                            .get(&module_path)
                            .is_some_and(|enums| enums.contains(local_name))
                    {
                        symbols
                            .module_type_exports
                            .entry(module_path.clone())
                            .or_default()
                            .insert(
                                specifier.exported_name().to_owned(),
                                TypeTarget {
                                    module_path: module_path.clone(),
                                    name: local_name.to_owned(),
                                },
                            );
                    }
                }
            }
        }
    }
}

fn bind_reexported_type_exports(syntax_trees: &[SyntaxTree], symbols: &mut SymbolTable) {
    let mut changed = true;
    while changed {
        changed = false;
        for syntax_tree in syntax_trees {
            let module_path = syntax_tree.source_file().path();
            let module_path_key = module_resolver::normalize_path(module_path);
            for statement in syntax_tree.program().statements() {
                match statement.declaration() {
                    Statement::ExportAll(export) => {
                        let Some(exported_tree) = module_resolver::resolve_in_compilation(
                            module_path,
                            export.module_specifier(),
                            syntax_trees,
                        ) else {
                            continue;
                        };
                        let exported_path =
                            module_resolver::normalize_path(exported_tree.source_file().path());
                        let exported_types = symbols
                            .module_type_exports
                            .get(&exported_path)
                            .cloned()
                            .unwrap_or_default();
                        let exports = symbols
                            .module_type_exports
                            .entry(module_path_key.clone())
                            .or_default();
                        for (name, target) in exported_types {
                            if name != "default" && !exports.contains_key(&name) {
                                exports.insert(name, target);
                                changed = true;
                            }
                        }
                    }
                    Statement::ExportNamedFrom(export) => {
                        let Some(exported_tree) = module_resolver::resolve_in_compilation(
                            module_path,
                            export.module_specifier(),
                            syntax_trees,
                        ) else {
                            continue;
                        };
                        let exported_path =
                            module_resolver::normalize_path(exported_tree.source_file().path());
                        let exported_types = symbols
                            .module_type_exports
                            .get(&exported_path)
                            .cloned()
                            .unwrap_or_default();
                        let exports = symbols
                            .module_type_exports
                            .entry(module_path_key.clone())
                            .or_default();
                        for specifier in export.specifiers() {
                            if let Some(target) = exported_types.get(specifier.local_name())
                                && !exports.contains_key(specifier.exported_name())
                            {
                                exports
                                    .insert(specifier.exported_name().to_owned(), target.clone());
                                changed = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn bind_imported_types(syntax_trees: &[SyntaxTree], symbols: &mut SymbolTable) {
    for syntax_tree in syntax_trees {
        let importer_path = syntax_tree.source_file().path();
        let importer_path_key = module_resolver::normalize_path(importer_path);
        for statement in syntax_tree.program().statements() {
            let Statement::ImportDeclaration(import) = statement.declaration() else {
                continue;
            };
            let Some(imported_tree) = module_resolver::resolve_in_compilation(
                importer_path,
                import.module_specifier(),
                syntax_trees,
            ) else {
                continue;
            };
            let imported_path = module_resolver::normalize_path(imported_tree.source_file().path());
            let Some(exports) = symbols.module_type_exports.get(&imported_path) else {
                continue;
            };
            let imported_types = exports
                .iter()
                .filter_map(|(exported_name, target)| {
                    import
                        .named_imports()
                        .iter()
                        .find(|specifier| specifier.imported_name() == exported_name)
                        .map(|specifier| (specifier.local_name().to_owned(), target.clone()))
                })
                .collect::<Vec<_>>();
            if !imported_types.is_empty() {
                let imports = symbols
                    .imported_types
                    .entry(importer_path_key.clone())
                    .or_default();
                imports.extend(imported_types);
            }
        }
    }
}

type LocalValues = HashMap<PathBuf, HashMap<String, Vec<String>>>;
type DeclaredExports = HashMap<PathBuf, Vec<(String, String)>>;

fn collect_local_values(
    syntax_trees: &[SyntaxTree],
    symbols: &SymbolTable,
) -> (LocalValues, DeclaredExports) {
    let mut local_values = HashMap::<PathBuf, HashMap<String, Vec<String>>>::new();
    let mut declared_exports = HashMap::<PathBuf, Vec<(String, String)>>::new();
    for syntax_tree in syntax_trees {
        let module_path = module_resolver::normalize_path(syntax_tree.source_file().path());
        let scoped_symbols = symbols.for_source(&module_path);
        let module_values = local_values.entry(module_path.clone()).or_default();
        for statement in syntax_tree.program().statements() {
            let declaration = statement.declaration();
            if let Some(variable) = declaration.as_variable_declaration() {
                module_values.insert(
                    variable.name().to_owned(),
                    variable_types(variable, &scoped_symbols),
                );
                if statement.is_exported() {
                    declared_exports
                        .entry(module_path.clone())
                        .or_default()
                        .push((variable.name().to_owned(), variable.name().to_owned()));
                }
            }
            if let Some(function) = declaration.as_function_declaration() {
                module_values.insert(
                    function.name().to_owned(),
                    function.return_type().map_or_else(
                        || vec!["any".to_owned()],
                        |ty| scoped_symbols.resolve_annotation(ty),
                    ),
                );
                if statement.is_exported() {
                    declared_exports
                        .entry(module_path.clone())
                        .or_default()
                        .push((function.name().to_owned(), function.name().to_owned()));
                }
            }
            if let Some(enum_declaration) = declaration.as_enum_declaration() {
                module_values.insert(
                    enum_declaration.name().to_owned(),
                    vec![format!("typeof {}", enum_declaration.name())],
                );
                if statement.is_exported() {
                    declared_exports
                        .entry(module_path.clone())
                        .or_default()
                        .push((
                            enum_declaration.name().to_owned(),
                            enum_declaration.name().to_owned(),
                        ));
                }
            }
            if let Statement::ExportDefault(expression) = declaration {
                module_values.insert("default".to_owned(), infer_expression_types(expression));
                declared_exports
                    .entry(module_path.clone())
                    .or_default()
                    .push(("default".to_owned(), "default".to_owned()));
            }
            if let Statement::ExportNamed(specifiers) = declaration {
                let exports = declared_exports.entry(module_path.clone()).or_default();
                exports.extend(specifiers.iter().map(|specifier| {
                    (
                        specifier.exported_name().to_owned(),
                        specifier.local_name().to_owned(),
                    )
                }));
            }
        }
    }

    (local_values, declared_exports)
}

fn resolve_module_exports(
    local_values: &LocalValues,
    declared_exports: DeclaredExports,
) -> HashMap<PathBuf, HashMap<String, Vec<String>>> {
    declared_exports
        .into_iter()
        .map(|(module_path, exports)| {
            let module_values = local_values.get(&module_path);
            let exported_values = exports
                .into_iter()
                .filter_map(|(exported_name, local_name)| {
                    module_values
                        .and_then(|values| values.get(&local_name))
                        .map(|types| (exported_name, types.clone()))
                })
                .collect();
            (module_path, exported_values)
        })
        .collect()
}

fn bind_imported_values(syntax_trees: &[SyntaxTree], symbols: &mut SymbolTable) {
    for syntax_tree in syntax_trees {
        let importer_path = syntax_tree.source_file().path();
        let importing_path_key = module_resolver::normalize_path(importer_path);
        for statement in syntax_tree.program().statements() {
            let Statement::ImportDeclaration(import) = statement.declaration() else {
                continue;
            };
            if import.is_type_only() {
                continue;
            }
            let imports = symbols
                .imported_values
                .entry(importing_path_key.clone())
                .or_default();
            for specifier in import.imported_bindings() {
                imports
                    .entry(specifier.local_name().to_owned())
                    .or_insert_with(|| vec!["any".to_owned()]);
            }

            let Some(imported_tree) = module_resolver::resolve_in_compilation(
                importer_path,
                import.module_specifier(),
                syntax_trees,
            ) else {
                continue;
            };
            let imported_path_key =
                module_resolver::normalize_path(imported_tree.source_file().path());
            let Some(exports) = symbols.module_exports.get(&imported_path_key) else {
                continue;
            };
            let imports = symbols
                .imported_values
                .entry(importing_path_key.clone())
                .or_default();
            for specifier in import.imported_bindings() {
                if let Some(types) = exports.get(specifier.imported_name()) {
                    imports.insert(specifier.local_name().to_owned(), types.clone());
                }
            }
            if let Some(namespace_import) = import.namespace_import() {
                for (exported_name, types) in exports {
                    imports.insert(
                        format!("{}.{}", namespace_import.local_name(), exported_name),
                        types.clone(),
                    );
                }
            }
        }
    }
}

fn bind_reexported_values(syntax_trees: &[SyntaxTree], symbols: &mut SymbolTable) {
    let mut changed = true;
    while changed {
        changed = false;
        for syntax_tree in syntax_trees {
            let importing_path = syntax_tree.source_file().path();
            let importing_path_key = module_resolver::normalize_path(importing_path);
            for statement in syntax_tree.program().statements() {
                match statement.declaration() {
                    Statement::ExportAll(export) => {
                        let Some(exported_tree) = module_resolver::resolve_in_compilation(
                            importing_path,
                            export.module_specifier(),
                            syntax_trees,
                        ) else {
                            continue;
                        };
                        let exported_path_key =
                            module_resolver::normalize_path(exported_tree.source_file().path());
                        let exported_values = symbols
                            .module_exports
                            .get(&exported_path_key)
                            .cloned()
                            .unwrap_or_default();
                        let exports = symbols
                            .module_exports
                            .entry(importing_path_key.clone())
                            .or_default();
                        for (name, types) in exported_values {
                            if name != "default" && !exports.contains_key(&name) {
                                exports.insert(name, types);
                                changed = true;
                            }
                        }
                    }
                    Statement::ExportNamedFrom(export) => {
                        if export.is_type_only() {
                            continue;
                        }
                        let Some(exported_tree) = module_resolver::resolve_in_compilation(
                            importing_path,
                            export.module_specifier(),
                            syntax_trees,
                        ) else {
                            continue;
                        };
                        let exported_path_key =
                            module_resolver::normalize_path(exported_tree.source_file().path());
                        let exported_values = symbols
                            .module_exports
                            .get(&exported_path_key)
                            .cloned()
                            .unwrap_or_default();
                        let exports = symbols
                            .module_exports
                            .entry(importing_path_key.clone())
                            .or_default();
                        for specifier in export.specifiers() {
                            if let Some(types) = exported_values.get(specifier.local_name())
                                && !exports.contains_key(specifier.exported_name())
                            {
                                exports.insert(specifier.exported_name().to_owned(), types.clone());
                                changed = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn variable_types(
    declaration: &crate::syntax::VariableDeclaration,
    symbols: &ScopedSymbolTable<'_>,
) -> Vec<String> {
    if let Some(annotation) = declaration.type_annotation() {
        return symbols.resolve_annotation(annotation);
    }
    declaration
        .initializer()
        .map_or_else(|| vec!["any".to_owned()], infer_expression_types)
}

fn infer_expression_types(expression: &Expression) -> Vec<String> {
    match expression {
        Expression::NumberLiteral { value, .. } => {
            vec![crate::type_system::numeric_literal_type(value).to_owned()]
        }
        Expression::UnaryExpression {
            operator: crate::syntax::UnaryOperator::Plus | crate::syntax::UnaryOperator::Negate,
            ..
        } => vec!["number".to_owned()],
        Expression::BooleanLiteral { .. }
        | Expression::UnaryExpression {
            operator: crate::syntax::UnaryOperator::LogicalNot,
            ..
        } => vec!["boolean".to_owned()],
        Expression::NullLiteral { .. } => vec!["null".to_owned()],
        Expression::StringLiteral { .. }
        | Expression::UnaryExpression {
            operator: crate::syntax::UnaryOperator::TypeOf,
            ..
        } => vec!["string".to_owned()],
        Expression::TypeAssertionExpression {
            type_annotation, ..
        } => type_annotation.type_spellings().collect(),
        Expression::BinaryExpression {
            operator: crate::syntax::BinaryOperator::NullishCoalesce,
            left,
            right,
            ..
        } => infer_merged_expression_types(left, right, true),
        Expression::BinaryExpression {
            left,
            operator:
                crate::syntax::BinaryOperator::LogicalAnd | crate::syntax::BinaryOperator::LogicalOr,
            right,
            ..
        } => infer_merged_expression_types(left, right, false),
        Expression::ConditionalExpression {
            when_true,
            when_false,
            ..
        } => infer_merged_expression_types(when_true, when_false, false),
        Expression::BinaryExpression { operator, .. }
            if matches!(
                *operator,
                crate::syntax::BinaryOperator::LessThan
                    | crate::syntax::BinaryOperator::GreaterThan
            ) =>
        {
            vec!["boolean".to_owned()]
        }
        Expression::BinaryExpression {
            left,
            operator,
            right,
            ..
        } if matches!(
            *operator,
            crate::syntax::BinaryOperator::Add
                | crate::syntax::BinaryOperator::Subtract
                | crate::syntax::BinaryOperator::Multiply
                | crate::syntax::BinaryOperator::Divide
                | crate::syntax::BinaryOperator::Remainder
        ) =>
        {
            infer_arithmetic_expression_types(*operator, left, right)
        }
        Expression::ArrayLiteral { elements, .. } => infer_array_expression_types(elements),
        Expression::AssignmentExpression {
            right,
            operator: crate::syntax::AssignmentOperator::Assign,
            ..
        } => infer_expression_types(right),
        Expression::AssignmentExpression {
            left,
            right,
            operator: crate::syntax::AssignmentOperator::AddAssign,
            ..
        } => infer_add_assignment_types(left, right),
        Expression::NewExpression { constructor, .. } => match constructor.as_ref() {
            Expression::Identifier { name, .. } => vec![name.clone()],
            _ => vec!["any".to_owned()],
        },
        Expression::Identifier { .. }
        | Expression::BinaryExpression { .. }
        | Expression::ParenthesizedExpression { .. }
        | Expression::CallExpression { .. }
        | Expression::ArrowFunction { .. }
        | Expression::PropertyAccessExpression { .. }
        | Expression::ElementAccessExpression { .. }
        | Expression::ObjectLiteral { .. } => vec!["any".to_owned()],
    }
}

fn infer_arithmetic_expression_types(
    operator: crate::syntax::BinaryOperator,
    left: &Expression,
    right: &Expression,
) -> Vec<String> {
    let left_types = infer_expression_types(left);
    let right_types = infer_expression_types(right);
    let result_type = if operator == crate::syntax::BinaryOperator::Add
        && (left_types.iter().any(|name| name == "string")
            || right_types.iter().any(|name| name == "string"))
    {
        "string"
    } else if left_types.iter().all(|name| name == "bigint")
        && right_types.iter().all(|name| name == "bigint")
    {
        "bigint"
    } else if left_types.iter().all(|name| name == "number")
        && right_types.iter().all(|name| name == "number")
    {
        "number"
    } else {
        "any"
    };
    vec![result_type.to_owned()]
}

fn infer_merged_expression_types(
    left: &Expression,
    right: &Expression,
    exclude_nullish_left_types: bool,
) -> Vec<String> {
    let left_types = infer_expression_types(left)
        .into_iter()
        .filter(|type_name| {
            !exclude_nullish_left_types || !matches!(type_name.as_str(), "null" | "undefined")
        });
    let mut result_types = Vec::new();
    for type_name in left_types.chain(infer_expression_types(right)) {
        if type_name == "any" {
            return vec!["any".to_owned()];
        }
        if !result_types.contains(&type_name) {
            result_types.push(type_name);
        }
    }
    result_types
}

fn infer_add_assignment_types(left: &Expression, right: &Expression) -> Vec<String> {
    let operand_types = infer_expression_types(left)
        .into_iter()
        .chain(infer_expression_types(right))
        .collect::<Vec<_>>();
    if operand_types.iter().any(|type_name| type_name == "string") {
        vec!["string".to_owned()]
    } else if operand_types.iter().any(|type_name| type_name == "any") {
        vec!["any".to_owned()]
    } else {
        vec!["number".to_owned()]
    }
}

fn infer_array_expression_types(elements: &[Expression]) -> Vec<String> {
    let element_types = elements
        .iter()
        .flat_map(infer_expression_types)
        .collect::<Vec<_>>();
    vec![crate::type_system::array_type(element_types)]
}
