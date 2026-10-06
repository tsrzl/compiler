//! Type creation and interning, modeled on the type constructors of TypeScript-Go's checker.

use std::collections::HashMap;

use super::compare::compare_types;
use super::types::{LiteralValue, Type, TypeData, TypeId};
use super::{ObjectFlags, TypeFlags};

/// The built-in types every compilation creates, in TypeScript-Go's creation order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    missing_docs,
    reason = "each field is named after the TypeScript-Go intrinsic it holds"
)]
pub struct Intrinsics {
    pub any: TypeId,
    pub auto: TypeId,
    pub wildcard: TypeId,
    pub blocked_string: TypeId,
    pub error: TypeId,
    pub unresolved: TypeId,
    pub non_inferrable_any: TypeId,
    pub intrinsic_marker: TypeId,
    pub unknown: TypeId,
    pub undefined: TypeId,
    pub undefined_widening: TypeId,
    pub missing: TypeId,
    pub optional: TypeId,
    pub null: TypeId,
    pub null_widening: TypeId,
    pub string: TypeId,
    pub number: TypeId,
    pub bigint: TypeId,
    pub regular_false: TypeId,
    pub false_type: TypeId,
    pub regular_true: TypeId,
    pub true_type: TypeId,
    pub boolean: TypeId,
    pub es_symbol: TypeId,
    pub void: TypeId,
    pub never: TypeId,
    pub silent_never: TypeId,
    pub implicit_never: TypeId,
    pub unreachable_never: TypeId,
    pub non_primitive: TypeId,
    pub string_or_number: TypeId,
    pub string_number_symbol: TypeId,
    pub number_or_bigint: TypeId,
}

/// The types of one compilation, interned so equal unions and literals share an identity.
#[derive(Debug, Clone)]
pub struct TypeTable {
    types: Vec<Type>,
    strict_null_checks: bool,
    intrinsics: Option<Intrinsics>,
    string_literals: HashMap<Box<str>, TypeId>,
    number_literals: HashMap<u64, TypeId>,
    nan_literal: Option<TypeId>,
    unions: HashMap<(Vec<TypeId>, Option<TypeId>), TypeId>,
}

impl TypeTable {
    /// Creates a table holding the intrinsic types.
    ///
    /// Without `strict_null_checks`, `null` and `undefined` widen to `any` and are absorbed by
    /// other union constituents.
    #[must_use]
    pub fn new(strict_null_checks: bool) -> Self {
        let mut table = Self {
            types: Vec::new(),
            strict_null_checks,
            intrinsics: None,
            string_literals: HashMap::new(),
            number_literals: HashMap::new(),
            nan_literal: None,
            unions: HashMap::new(),
        };
        let intrinsics = table.create_intrinsics();
        table.intrinsics = Some(intrinsics);
        table
    }

    fn create_intrinsics(&mut self) -> Intrinsics {
        let any = self.intrinsic(TypeFlags::ANY, "any");
        let auto = self.intrinsic_with(TypeFlags::ANY, "any", ObjectFlags::NON_INFERRABLE_TYPE);
        let wildcard = self.intrinsic(TypeFlags::ANY, "any");
        let blocked_string = self.intrinsic(TypeFlags::ANY, "any");
        let error = self.intrinsic(TypeFlags::ANY, "error");
        let unresolved = self.intrinsic(TypeFlags::ANY, "unresolved");
        let non_inferrable_any =
            self.intrinsic_with(TypeFlags::ANY, "any", ObjectFlags::CONTAINS_WIDENING_TYPE);
        let intrinsic_marker = self.intrinsic(TypeFlags::ANY, "intrinsic");
        let unknown = self.intrinsic(TypeFlags::UNKNOWN, "unknown");
        let undefined = self.intrinsic(TypeFlags::UNDEFINED, "undefined");
        let undefined_widening = self.widening_type(undefined, "undefined");
        let missing = self.intrinsic(TypeFlags::UNDEFINED, "undefined");
        let optional = self.intrinsic(TypeFlags::UNDEFINED, "undefined");
        let null = self.intrinsic(TypeFlags::NULL, "null");
        let null_widening = self.widening_type(null, "null");
        let string = self.intrinsic(TypeFlags::STRING, "string");
        let number = self.intrinsic(TypeFlags::NUMBER, "number");
        let bigint = self.intrinsic(TypeFlags::BIG_INT, "bigint");
        let (regular_false, false_type) = self.boolean_literal_pair(false);
        let (regular_true, true_type) = self.boolean_literal_pair(true);
        // TypeScript-Go creates `boolean` here, before the remaining intrinsics. Union creation
        // consults only the intrinsics created so far, so the later ones start as `any`
        // placeholders and are filled in as they are created.
        self.intrinsics = Some(Intrinsics {
            any,
            auto,
            wildcard,
            blocked_string,
            error,
            unresolved,
            non_inferrable_any,
            intrinsic_marker,
            unknown,
            undefined,
            undefined_widening,
            missing,
            optional,
            null,
            null_widening,
            string,
            number,
            bigint,
            regular_false,
            false_type,
            regular_true,
            true_type,
            boolean: any,
            es_symbol: any,
            void: any,
            never: any,
            silent_never: any,
            implicit_never: any,
            unreachable_never: any,
            non_primitive: any,
            string_or_number: any,
            string_number_symbol: any,
            number_or_bigint: any,
        });
        let boolean = self.union_type(&[regular_false, regular_true]);
        let es_symbol = self.intrinsic(TypeFlags::ES_SYMBOL, "symbol");
        let void = self.intrinsic(TypeFlags::VOID, "void");
        let never = self.intrinsic(TypeFlags::NEVER, "never");
        let silent_never =
            self.intrinsic_with(TypeFlags::NEVER, "never", ObjectFlags::NON_INFERRABLE_TYPE);
        let implicit_never = self.intrinsic(TypeFlags::NEVER, "never");
        let unreachable_never = self.intrinsic(TypeFlags::NEVER, "never");
        let non_primitive = self.intrinsic(TypeFlags::NON_PRIMITIVE, "object");
        let partial = Intrinsics {
            boolean,
            es_symbol,
            void,
            never,
            silent_never,
            implicit_never,
            unreachable_never,
            non_primitive,
            ..*self.intrinsics()
        };
        self.intrinsics = Some(partial);
        Intrinsics {
            string_or_number: self.union_type(&[string, number]),
            string_number_symbol: self.union_type(&[string, number, es_symbol]),
            number_or_bigint: self.union_type(&[number, bigint]),
            ..partial
        }
    }

    /// Returns the intrinsic types.
    ///
    /// # Panics
    ///
    /// Panics only while the intrinsics themselves are being created.
    #[must_use]
    pub fn intrinsics(&self) -> &Intrinsics {
        self.intrinsics
            .as_ref()
            .expect("intrinsics are created with the table")
    }

    /// Returns the type with identity `id`.
    ///
    /// # Panics
    ///
    /// Panics when `id` was not produced by this table.
    #[must_use]
    pub fn get(&self, id: TypeId) -> &Type {
        &self.types[id.index()]
    }

    /// Returns the constituents of a union type, or the type itself otherwise.
    #[must_use]
    pub fn union_constituents(&self, id: TypeId) -> Vec<TypeId> {
        match self.get(id).data() {
            TypeData::Union { types, .. } => types.clone(),
            _ => vec![id],
        }
    }

    fn create(&mut self, flags: TypeFlags, object_flags: ObjectFlags, data: TypeData) -> TypeId {
        let id = TypeId::from_index(self.types.len());
        self.types.push(Type {
            flags,
            // Computed-on-demand flags are never inherited from a constructor's input.
            object_flags: object_flags.without(
                ObjectFlags::COULD_CONTAIN_TYPE_VARIABLES_COMPUTED
                    | ObjectFlags::COULD_CONTAIN_TYPE_VARIABLES
                    | ObjectFlags::MEMBERS_RESOLVED,
            ),
            data,
        });
        id
    }

    fn intrinsic(&mut self, flags: TypeFlags, name: &'static str) -> TypeId {
        self.intrinsic_with(flags, name, ObjectFlags::NONE)
    }

    fn intrinsic_with(
        &mut self,
        flags: TypeFlags,
        name: &'static str,
        object_flags: ObjectFlags,
    ) -> TypeId {
        self.create(flags, object_flags, TypeData::Intrinsic { name })
    }

    /// Returns a variant of `null` or `undefined` that widens to `any` without strict null checks.
    fn widening_type(&mut self, non_widening: TypeId, name: &'static str) -> TypeId {
        if self.strict_null_checks {
            return non_widening;
        }
        let flags = self.get(non_widening).flags();
        self.intrinsic_with(flags, name, ObjectFlags::CONTAINS_WIDENING_TYPE)
    }

    fn boolean_literal_pair(&mut self, value: bool) -> (TypeId, TypeId) {
        let regular = self.literal(
            TypeFlags::BOOLEAN_LITERAL,
            LiteralValue::Boolean(value),
            None,
        );
        let fresh = self.literal(
            TypeFlags::BOOLEAN_LITERAL,
            LiteralValue::Boolean(value),
            Some(regular),
        );
        self.set_fresh_type(regular, fresh);
        self.set_fresh_type(fresh, fresh);
        (regular, fresh)
    }

    /// Creates a literal type whose regular variant is `regular_type`, or itself.
    fn literal(
        &mut self,
        flags: TypeFlags,
        value: LiteralValue,
        regular_type: Option<TypeId>,
    ) -> TypeId {
        let id = TypeId::from_index(self.types.len());
        self.create(
            flags,
            ObjectFlags::NONE,
            TypeData::Literal {
                value,
                fresh_type: None,
                regular_type: regular_type.unwrap_or(id),
            },
        )
    }

    fn set_fresh_type(&mut self, id: TypeId, fresh: TypeId) {
        if let TypeData::Literal { fresh_type, .. } = &mut self.types[id.index()].data {
            *fresh_type = Some(fresh);
        }
    }

    /// Returns the interned regular string literal type for `value`.
    pub fn string_literal_type(&mut self, value: &str) -> TypeId {
        if let Some(&id) = self.string_literals.get(value) {
            return id;
        }
        let id = self.literal(
            TypeFlags::STRING_LITERAL,
            LiteralValue::String(value.into()),
            None,
        );
        self.string_literals.insert(value.into(), id);
        id
    }

    /// Returns the interned regular number literal type for `value`; all NaN values share one.
    pub fn number_literal_type(&mut self, value: f64) -> TypeId {
        if value.is_nan() {
            if let Some(id) = self.nan_literal {
                return id;
            }
            let id = self.literal(TypeFlags::NUMBER_LITERAL, LiteralValue::Number(value), None);
            self.nan_literal = Some(id);
            return id;
        }
        let key = value.to_bits();
        if let Some(&id) = self.number_literals.get(&key) {
            return id;
        }
        let id = self.literal(TypeFlags::NUMBER_LITERAL, LiteralValue::Number(value), None);
        self.number_literals.insert(key, id);
        id
    }

    /// Returns the fresh variant of a literal type, as written in an expression; other types are
    /// returned unchanged.
    pub fn fresh_literal_type(&mut self, id: TypeId) -> TypeId {
        let flags = self.get(id).flags();
        let TypeData::Literal {
            value, fresh_type, ..
        } = self.get(id).data().clone()
        else {
            return id;
        };
        if !flags.intersects(TypeFlags::FRESHABLE) {
            return id;
        }
        if let Some(fresh) = fresh_type {
            return fresh;
        }
        let fresh = self.literal(flags, value, Some(id));
        self.set_fresh_type(fresh, fresh);
        self.set_fresh_type(id, fresh);
        fresh
    }

    fn is_fresh_literal_type(&self, id: TypeId) -> bool {
        self.get(id).flags().intersects(TypeFlags::FRESHABLE)
            && matches!(self.get(id).data(), TypeData::Literal { fresh_type: Some(fresh), .. } if *fresh == id)
    }

    fn regular_type_of_literal(&self, id: TypeId) -> TypeId {
        match self.get(id).data() {
            TypeData::Literal { regular_type, .. } => *regular_type,
            _ => id,
        }
    }

    /// Returns the union of `types` with literal reduction: duplicates, `never`, and literals
    /// subsumed by their primitive or regular type are removed, and `any` or `unknown` absorb
    /// the union.
    pub fn union_type(&mut self, types: &[TypeId]) -> TypeId {
        match types {
            [] => self.intrinsics().never,
            [only] => *only,
            _ => self.union_type_worker(types),
        }
    }

    fn union_type_worker(&mut self, types: &[TypeId]) -> TypeId {
        let mut type_set = Vec::with_capacity(types.len());
        let includes = self.add_types_to_union(&mut type_set, TypeFlags::NONE, types);
        let intrinsics = *self.intrinsics();
        if includes.intersects(TypeFlags::ANY_OR_UNKNOWN) {
            if includes.intersects(TypeFlags::ANY) {
                return if includes.intersects(TypeFlags::INCLUDES_WILDCARD) {
                    intrinsics.wildcard
                } else if includes.intersects(TypeFlags::INCLUDES_ERROR) {
                    intrinsics.error
                } else {
                    intrinsics.any
                };
            }
            return intrinsics.unknown;
        }
        // A union with both `undefined` and the missing-property type keeps only `undefined`.
        if includes.intersects(TypeFlags::UNDEFINED)
            && type_set.len() >= 2
            && type_set[0] == intrinsics.undefined
            && type_set[1] == intrinsics.missing
        {
            type_set.remove(1);
        }
        if includes.intersects(
            TypeFlags::ENUM
                | TypeFlags::LITERAL
                | TypeFlags::UNIQUE_ES_SYMBOL
                | TypeFlags::TEMPLATE_LITERAL
                | TypeFlags::STRING_MAPPING,
        ) || includes.intersects(TypeFlags::VOID) && includes.intersects(TypeFlags::UNDEFINED)
        {
            self.remove_redundant_literal_types(&mut type_set, includes);
        }
        if type_set.is_empty() {
            return if includes.intersects(TypeFlags::NULL) {
                if includes.intersects(TypeFlags::INCLUDES_NON_WIDENING_TYPE) {
                    intrinsics.null
                } else {
                    intrinsics.null_widening
                }
            } else if includes.intersects(TypeFlags::UNDEFINED) {
                if includes.intersects(TypeFlags::INCLUDES_NON_WIDENING_TYPE) {
                    intrinsics.undefined
                } else {
                    intrinsics.undefined_widening
                }
            } else {
                intrinsics.never
            };
        }
        let mut object_flags = ObjectFlags::NONE;
        if !includes.intersects(TypeFlags::NOT_PRIMITIVE_UNION) {
            object_flags |= ObjectFlags::PRIMITIVE_UNION;
        }
        if includes.intersects(TypeFlags::INTERSECTION) {
            object_flags |= ObjectFlags::CONTAINS_INTERSECTIONS;
        }
        self.union_type_from_sorted_list(type_set, object_flags)
    }

    fn add_types_to_union(
        &self,
        type_set: &mut Vec<TypeId>,
        mut includes: TypeFlags,
        types: &[TypeId],
    ) -> TypeFlags {
        let mut last = None;
        for &id in types {
            if last == Some(id) {
                continue;
            }
            if let TypeData::Union {
                types: constituents,
                origin,
            } = self.get(id).data()
            {
                if origin.is_some() {
                    includes |= TypeFlags::UNION;
                }
                includes = self.add_types_to_union(type_set, includes, constituents);
            } else {
                includes = self.add_type_to_union(type_set, includes, id);
            }
            last = Some(id);
        }
        includes
    }

    fn add_type_to_union(
        &self,
        type_set: &mut Vec<TypeId>,
        mut includes: TypeFlags,
        id: TypeId,
    ) -> TypeFlags {
        let ty = self.get(id);
        let flags = ty.flags();
        // `never` constituents are ignored.
        if flags.intersects(TypeFlags::NEVER) {
            return includes;
        }
        includes |= flags & TypeFlags::INCLUDES_MASK;
        if flags.intersects(TypeFlags::INSTANTIABLE) {
            includes |= TypeFlags::INCLUDES_INSTANTIABLE;
        }
        if flags.intersects(TypeFlags::INTERSECTION)
            && ty
                .object_flags()
                .intersects(ObjectFlags::IS_CONSTRAINED_TYPE_VARIABLE)
        {
            includes |= TypeFlags::INCLUDES_CONSTRAINED_TYPE_VARIABLE;
        }
        let intrinsics = self.intrinsics();
        if id == intrinsics.wildcard {
            includes |= TypeFlags::INCLUDES_WILDCARD;
        }
        if id == intrinsics.error {
            includes |= TypeFlags::INCLUDES_ERROR;
        }
        if !self.strict_null_checks && flags.intersects(TypeFlags::NULLABLE) {
            if !ty
                .object_flags()
                .intersects(ObjectFlags::CONTAINS_WIDENING_TYPE)
            {
                includes |= TypeFlags::INCLUDES_NON_WIDENING_TYPE;
            }
        } else if let Err(position) =
            type_set.binary_search_by(|&probe| compare_types(self, probe, id))
        {
            type_set.insert(position, id);
        }
        includes
    }

    fn remove_redundant_literal_types(&self, types: &mut Vec<TypeId>, includes: TypeFlags) {
        let mut index = types.len();
        while index > 0 {
            index -= 1;
            let id = types[index];
            let flags = self.get(id).flags();
            let remove = flags.intersects(
                TypeFlags::STRING_LITERAL | TypeFlags::TEMPLATE_LITERAL | TypeFlags::STRING_MAPPING,
            ) && includes.intersects(TypeFlags::STRING)
                || flags.intersects(TypeFlags::NUMBER_LITERAL)
                    && includes.intersects(TypeFlags::NUMBER)
                || flags.intersects(TypeFlags::BIG_INT_LITERAL)
                    && includes.intersects(TypeFlags::BIG_INT)
                || flags.intersects(TypeFlags::UNIQUE_ES_SYMBOL)
                    && includes.intersects(TypeFlags::ES_SYMBOL)
                || self.is_fresh_literal_type(id)
                    && self.contains_type(types, self.regular_type_of_literal(id));
            if remove {
                types.remove(index);
            }
        }
    }

    fn contains_type(&self, types: &[TypeId], id: TypeId) -> bool {
        types
            .binary_search_by(|&probe| compare_types(self, probe, id))
            .is_ok()
    }

    /// Returns the interned union of constituents already in [`compare_types`] order.
    fn union_type_from_sorted_list(
        &mut self,
        types: Vec<TypeId>,
        object_flags: ObjectFlags,
    ) -> TypeId {
        match types.as_slice() {
            [] => return self.intrinsics().never,
            [only] => return *only,
            _ => {}
        }
        let key = (types, None);
        if let Some(&id) = self.unions.get(&key) {
            return id;
        }
        let types = key.0.clone();
        let propagating = types
            .iter()
            .filter(|&&id| !self.get(id).flags().intersects(TypeFlags::NULLABLE))
            .fold(ObjectFlags::NONE, |flags, &id| {
                flags | self.get(id).object_flags()
            })
            & ObjectFlags::PROPAGATING_FLAGS;
        let mut flags = TypeFlags::UNION;
        if let [first, second] = types.as_slice()
            && self
                .get(*first)
                .flags()
                .intersects(TypeFlags::BOOLEAN_LITERAL)
            && self
                .get(*second)
                .flags()
                .intersects(TypeFlags::BOOLEAN_LITERAL)
        {
            flags |= TypeFlags::BOOLEAN;
        }
        let id = self.create(
            flags,
            object_flags | propagating,
            TypeData::Union {
                types,
                origin: None,
            },
        );
        self.unions.insert(key, id);
        id
    }
}
