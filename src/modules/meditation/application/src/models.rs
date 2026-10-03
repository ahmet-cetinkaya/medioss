use std::collections::HashMap;

use crate::ports::{
    Category, CategoryRepository, MembershipRepository, Program, ProgramRepository,
};
use crate::search;
use crate::ApplicationError;

pub struct SearchIndex {
    programs: HashMap<i64, String>,
    categories: HashMap<i64, String>,
    program_ids_by_meditation: HashMap<i64, Vec<i64>>,
    category_ids_by_program: HashMap<i64, i64>,
}

impl SearchIndex {
    pub async fn build<M, P, C>(
        memberships: &M,
        programs: &P,
        categories: &C,
    ) -> Result<Self, ApplicationError>
    where
        M: MembershipRepository + ?Sized,
        P: ProgramRepository + ?Sized,
        C: CategoryRepository + ?Sized,
    {
        let programs_list: Vec<Program> = programs
            .list_all()
            .await
            .map_err(ApplicationError::Repository)?;
        let categories_list: Vec<Category> = categories
            .list_all()
            .await
            .map_err(ApplicationError::Repository)?;
        let mut index = SearchIndex {
            programs: programs_list
                .iter()
                .map(|program| (program.id, search::fold(&program.title)))
                .collect(),
            categories: categories_list
                .iter()
                .map(|category| (category.id, search::fold(&category.name)))
                .collect(),
            program_ids_by_meditation: HashMap::new(),
            category_ids_by_program: programs_list
                .iter()
                .filter_map(|program| {
                    program
                        .category_id
                        .map(|category_id| (program.id, category_id))
                })
                .collect(),
        };
        for program in &programs_list {
            for membership in memberships
                .list_by_program(program.id)
                .await
                .map_err(ApplicationError::Repository)?
            {
                index
                    .program_ids_by_meditation
                    .entry(membership.meditation_id)
                    .or_default()
                    .push(program.id);
            }
        }
        Ok(index)
    }

    pub fn haystack_for(&self, meditation: &meditation_domain::Meditation) -> String {
        let mut haystack = search::fold(meditation.name());
        if let Some(program_ids) = self.program_ids_by_meditation.get(&meditation.id()) {
            for program_id in program_ids {
                if let Some(name) = self.programs.get(program_id) {
                    haystack.push(' ');
                    haystack.push_str(name);
                }
                if let Some(category_id) = self.category_ids_by_program.get(program_id) {
                    if let Some(name) = self.categories.get(category_id) {
                        haystack.push(' ');
                        haystack.push_str(name);
                    }
                }
            }
        }
        haystack
    }
}
