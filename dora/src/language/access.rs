use crate::language::sem_analysis::{
    ClassDefinitionId, ConstDefinitionId, EnumDefinitionId, FctDefinitionId, FctParent, FieldId,
    GlobalDefinitionId, ModuleDefinitionId, SemAnalysis, TraitDefinitionId, ValueDefinitionFieldId,
    ValueDefinitionId, Visibility,
};
use crate::language::sym::Sym;

pub fn sym_accessible_from(sa: &SemAnalysis, sym: Sym, module_id: ModuleDefinitionId) -> bool {
    match sym {
        Sym::Annotation(_) => unimplemented!(),
        Sym::Class(class_id) => class_accessible_from(sa, class_id, module_id),
        Sym::Const(const_id) => const_accessible_from(sa, const_id, module_id),
        Sym::Enum(enum_id) => enum_accessible_from(sa, enum_id, module_id),
        Sym::EnumVariant(_, _) => unreachable!(),
        Sym::Fct(fct_id) => fct_accessible_from(sa, fct_id, module_id),
        Sym::Field(_) => unreachable!(),
        Sym::Global(global_id) => global_accessible_from(sa, global_id, module_id),
        Sym::Module(sym_module_id) => module_accessible_from(sa, sym_module_id, module_id),
        Sym::Value(value_id) => value_accessible_from(sa, value_id, module_id),
        Sym::Trait(trait_id) => trait_accessible_from(sa, trait_id, module_id),
        Sym::TypeParam(_) => unreachable!(),
        Sym::Var(_) => unreachable!(),
    }
}

pub fn global_accessible_from(
    sa: &SemAnalysis,
    global_id: GlobalDefinitionId,
    module_id: ModuleDefinitionId,
) -> bool {
    let global = sa.globals.idx(global_id);
    let global = global.read();

    accessible_from(sa, global.module_id, global.visibility, module_id)
}

pub fn class_accessible_from(
    sa: &SemAnalysis,
    cls_id: ClassDefinitionId,
    module_id: ModuleDefinitionId,
) -> bool {
    let cls = sa.classes.idx(cls_id);
    let cls = cls.read();

    accessible_from(sa, cls.module_id, cls.visibility, module_id)
}

pub fn class_field_accessible_from(
    sa: &SemAnalysis,
    cls_id: ClassDefinitionId,
    field_id: FieldId,
    module_id: ModuleDefinitionId,
) -> bool {
    let cls = sa.classes.idx(cls_id);
    let cls = cls.read();

    let field = &cls.fields[field_id];

    accessible_from(
        sa,
        cls.module_id,
        if cls.visibility.is_public() && field.visibility.is_public() {
            Visibility::Public
        } else {
            Visibility::Module
        },
        module_id,
    )
}

pub fn method_accessible_from(
    sa: &SemAnalysis,
    fct_id: FctDefinitionId,
    module_id: ModuleDefinitionId,
) -> bool {
    let fct = sa.fcts.idx(fct_id);
    let fct = fct.read();

    let element_visibility = match fct.parent {
        FctParent::Extension(_) => fct.visibility,
        FctParent::Impl(_) | FctParent::Trait(_) => {
            // TODO: This should probably be limited
            Visibility::Public
        }

        FctParent::Function(_) | FctParent::None => unreachable!(),
    };

    accessible_from(sa, fct.module_id, element_visibility, module_id)
}

pub fn fct_accessible_from(
    sa: &SemAnalysis,
    fct_id: FctDefinitionId,
    module_id: ModuleDefinitionId,
) -> bool {
    let fct = sa.fcts.idx(fct_id);
    let fct = fct.read();

    accessible_from(sa, fct.module_id, fct.visibility, module_id)
}

pub fn enum_accessible_from(
    sa: &SemAnalysis,
    enum_id: EnumDefinitionId,
    module_id: ModuleDefinitionId,
) -> bool {
    let enum_ = sa.enums[enum_id].read();

    accessible_from(sa, enum_.module_id, enum_.visibility, module_id)
}

pub fn value_accessible_from(
    sa: &SemAnalysis,
    value_id: ValueDefinitionId,
    module_id: ModuleDefinitionId,
) -> bool {
    let value = sa.values.idx(value_id);
    let value = value.read();

    accessible_from(sa, value.module_id, value.visibility, module_id)
}

pub fn value_field_accessible_from(
    sa: &SemAnalysis,
    value_id: ValueDefinitionId,
    field_id: ValueDefinitionFieldId,
    module_id: ModuleDefinitionId,
) -> bool {
    let value = sa.values.idx(value_id);
    let value = value.read();

    let field = &value.fields[field_id.to_usize()];

    accessible_from(
        sa,
        value.module_id,
        if value.visibility.is_public() && field.visibility.is_public() {
            Visibility::Public
        } else {
            Visibility::Module
        },
        module_id,
    )
}

pub fn module_accessible_from(
    sa: &SemAnalysis,
    target_id: ModuleDefinitionId,
    from_id: ModuleDefinitionId,
) -> bool {
    accessible_from(sa, target_id, Visibility::Public, from_id)
}

pub fn trait_accessible_from(
    sa: &SemAnalysis,
    trait_id: TraitDefinitionId,
    module_id: ModuleDefinitionId,
) -> bool {
    let trait_ = sa.traits[trait_id].read();

    accessible_from(sa, trait_.module_id, trait_.visibility, module_id)
}

pub fn const_accessible_from(
    sa: &SemAnalysis,
    const_id: ConstDefinitionId,
    module_id: ModuleDefinitionId,
) -> bool {
    let const_ = sa.consts.idx(const_id);
    let const_ = const_.read();

    accessible_from(sa, const_.module_id, const_.visibility, module_id)
}

pub fn is_default_accessible(
    sa: &SemAnalysis,
    target_id: ModuleDefinitionId,
    from_id: ModuleDefinitionId,
) -> bool {
    // each module can access itself
    if target_id == from_id {
        return true;
    }

    // modules can access all their parents
    module_contains(sa, target_id, from_id)
}

fn accessible_from(
    sa: &SemAnalysis,
    target_module_id: ModuleDefinitionId,
    element_visibility: Visibility,
    user_module_id: ModuleDefinitionId,
) -> bool {
    // Each module can access stuff in itself.
    if target_module_id == user_module_id {
        return true;
    }

    // Modules can always access all their parents.
    if module_contains(sa, target_module_id, user_module_id) {
        return true;
    }

    // Find the common parent of both modules.
    let common_parent_id = common_parent(sa, target_module_id, user_module_id);

    let target_module = &sa.modules[target_module_id].read();

    {
        let target_module = sa.modules.idx(target_module_id);
        let target_module = target_module.read();

        let user_module = sa.modules.idx(user_module_id);
        let user_module = user_module.read();

        if target_module.package_id == user_module.package_id {
            assert!(common_parent_id.is_some());
        } else {
            assert!(common_parent_id.is_none());
        }
    }

    if let Some(common_parent_id) = common_parent_id {
        let common_parent_depth = sa.modules[common_parent_id].read().depth;

        // The common parent module is an ancestor of the user module, which means
        // the user module has access to everything along that path including the
        // common parent modules direct children.
        if common_parent_depth + 1 == target_module.depth {
            element_visibility.is_public()
        } else {
            let start_depth = common_parent_depth + 2;
            for &ns_id in &target_module.parents[start_depth..] {
                let ns = &sa.modules[ns_id].read();
                if !ns.visibility.is_public() {
                    return false;
                }
            }

            target_module.visibility.is_public() && element_visibility.is_public()
        }
    } else {
        // No common parent: means we try to access another package
        // the whole path needs to be public

        for &ns_id in &target_module.parents {
            let ns = &sa.modules[ns_id].read();
            if !ns.visibility.is_public() {
                return false;
            }
        }

        target_module.visibility.is_public() && element_visibility.is_public()
    }
}

fn common_parent(
    sa: &SemAnalysis,
    lhs_id: ModuleDefinitionId,
    rhs_id: ModuleDefinitionId,
) -> Option<ModuleDefinitionId> {
    if lhs_id == rhs_id {
        return Some(lhs_id);
    }

    let lhs = &sa.modules[lhs_id].read();
    let rhs = &sa.modules[rhs_id].read();

    if lhs.depth > rhs.depth {
        if lhs.parents[rhs.depth] == rhs_id {
            return Some(rhs_id);
        } else {
            // do nothing
        }
    } else if rhs.depth > lhs.depth {
        if rhs.parents[lhs.depth] == lhs_id {
            return Some(lhs_id);
        } else {
            // do nothing
        }
    }

    let start = std::cmp::min(lhs.depth, rhs.depth);

    for depth in (0..start).rev() {
        if lhs.parents[depth] == rhs.parents[depth] {
            return Some(lhs.parents[depth]);
        }
    }

    None
}

pub fn module_contains(
    sa: &SemAnalysis,
    parent_id: ModuleDefinitionId,
    child_id: ModuleDefinitionId,
) -> bool {
    if parent_id == child_id {
        return true;
    }

    let module = &sa.modules[child_id].read();
    module.parents.contains(&parent_id)
}
