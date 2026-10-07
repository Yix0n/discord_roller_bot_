use std::collections::HashMap;
use once_cell::sync::Lazy;
use rand::prelude::{IndexedRandom, ThreadRng};
use rand::random_range;
use crate::mechanics::npc_generator::data::traits::generate_traits;
use crate::mechanics::npc_generator::provider::NameProvider;
use crate::mechanics::npc_generator::types::{Gender, GenerateNpcParams, GeneratedName, NameSet, NpcGenerator, Race};

pub struct NameGenerator<P: NameProvider> {
    provider: P,
    cache: Lazy<HashMap<(Race, Gender), NameSet>>,
}

impl<P: NameProvider> NameGenerator<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            cache: Lazy::new(|| HashMap::new()),
        }
    }

    pub fn generate_npc(&mut self, params: GenerateNpcParams) -> NpcGenerator {
        let race = params.race.unwrap_or(Race::random());
        let gender = params.gender.unwrap_or(Gender::random());
        let traits_amount = params.trait_amount.unwrap_or(random_range(2..=6));

        let name_set = self.get_name_set(race.clone(), gender.clone());

        let mut rng = ThreadRng::default();

        let first_name = name_set.
            first_names
            .choose(&mut rng)
            .map(|s| s.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let last_name = name_set.
            last_names
            .choose(&mut rng)
            .map(|s| s.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let traits = generate_traits(traits_amount as usize);

        NpcGenerator {
            name: GeneratedName {
                first: first_name,
                last: last_name,
                race,
                gender,
            },
            traits,
        }
    }

    #[inline(always)]
    fn get_name_set(&mut self, race: Race, gender: Gender) -> &NameSet {
        let key = (race.clone(), gender.clone());
        self.cache.entry(key).or_insert_with(|| {
            self.provider.get_names(race, gender)
                .cloned()
                .unwrap_or_else(|| NameSet::new(vec!["Unknown".to_string()], vec!["Unknown".to_string()]))
        })
    }
}