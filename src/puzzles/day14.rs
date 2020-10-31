use anyhow::anyhow;
use anyhow::Error;
use thiserror::Error;

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::io::Read;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum Chemical {
    Ore,
    Named(String),
    Fuel,
}

impl FromStr for Chemical {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "ORE" => Chemical::Ore,
            "FUEL" => Chemical::Fuel,
            name => Chemical::Named(name.to_string()),
        })
    }
}

impl fmt::Display for Chemical {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chemical::Ore => write!(f, "ORE"),
            Chemical::Fuel => write!(f, "FUEL"),
            Chemical::Named(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Reactant {
    chemical: Chemical,
    quantity: usize,
}

impl Reactant {
    fn new(chemical: Chemical, quantity: usize) -> Self {
        Reactant { chemical, quantity }
    }

    fn fuel(quantity: usize) -> Self {
        Reactant::new(Chemical::Fuel, quantity)
    }
}

impl FromStr for Reactant {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut items = s.trim().split_ascii_whitespace();
        let quantity: usize = items
            .next()
            .ok_or(anyhow!("Missing quantity: {}", s))?
            .parse()?;
        let chemical: Chemical = items
            .next()
            .ok_or(anyhow!("Missing chemical: {}", s))?
            .parse()?;

        if items.next().is_some() {
            return Err(anyhow!("Too many items for chemical: {}", s));
        }

        Ok(Reactant { chemical, quantity })
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct Reaction {
    inputs: Vec<Reactant>,
    output: Reactant,
}

impl Reaction {
    fn output(&self) -> &Reactant {
        &self.output
    }

    fn repeat(&self, n: usize) -> Self {
        Reaction {
            inputs: self
                .inputs
                .iter()
                .map(|r| Reactant::new(r.chemical.clone(), r.quantity * n))
                .collect(),
            output: Reactant::new(self.output.chemical.clone(), self.output.quantity * n),
        }
    }
}

impl fmt::Debug for Reaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Reaction({}",
            self.inputs
                .iter()
                .map(|r| format!("{} {}", r.quantity, r.chemical))
                .collect::<Vec<String>>()
                .join(", "),
        )?;
        write!(f, " => ")?;
        write!(f, "{} {})", self.output.quantity, self.output.chemical)?;
        Ok(())
    }
}

impl FromStr for Reaction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split("=>");
        let inputs = {
            let text = parts.next().ok_or(anyhow!("Missing inputs!"))?;
            text.split(',')
                .map(|s| s.parse::<Reactant>())
                .collect::<Result<Vec<Reactant>, Error>>()?
        };
        let output = {
            let text = parts.next().ok_or(anyhow!("Missing outputs!"))?;
            let mut outputs = text.split(',').map(|s| s.parse::<Reactant>());
            let output = outputs.next().ok_or(anyhow!("Expected an output!"))??;
            if outputs.next().is_some() {
                return Err(anyhow!("Too many outputs: {}", s));
            }
            output
        };

        if parts.next().is_some() {
            return Err(anyhow!("Too much data to parse: {}", s));
        }

        Ok(Reaction { inputs, output })
    }
}

#[derive(Debug)]
struct Library {
    reactions: HashMap<Chemical, Reaction>,
}

impl Library {
    fn read(input: Box<dyn Read + 'static>) -> Result<Library, Error> {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(input);
        let mut items = HashMap::new();
        for line in reader.lines() {
            let reaction: Reaction = line?.parse()?;
            items.insert(reaction.output().chemical.clone(), reaction);
        }

        Ok(Library { reactions: items })
    }

    fn get(&self, chemical: &Chemical) -> Option<&Reaction> {
        self.reactions.get(chemical)
    }

    fn recipe(&self, target: Reactant) -> Result<Recipe, Error> {
        self.recipe_builder(target).map(|rb| rb.build())
    }

    fn recipe_builder(&self, target: Reactant) -> Result<RecipeBuilder, Error> {
        let mut recipe = RecipeBuilder::new(&target);

        recipe.add(
            self.get(&target.chemical)
                .ok_or(anyhow!("No recipe creates target {:?}", target))?
                .clone(),
        )?;

        while !recipe.is_finished() {
            let reaction = {
                if let Some(chemical) = recipe.chemical_requirement() {
                    self.get(&chemical)
                        .ok_or(anyhow!("No recipe creates chemical {:?}", chemical))?
                } else {
                    return Err(anyhow!("No requirements remain?"));
                }
            };
            recipe.add(reaction.clone())?;
        }

        Ok(recipe)
    }

    fn consume(&self, quantity: usize) -> Result<Recipe, Error> {
        let mut guess = quantity / self.recipe_builder(Reactant::fuel(1))?.ore_requirement();
        let mut incr = guess / 2;

        loop {
            let ore = self
                .recipe_builder(Reactant::fuel(guess + incr))?
                .ore_requirement();

            if ore > quantity {
                // When we are stepping by single values, we must be done.
                if incr == 1 {
                    break;
                } else {
                    incr = incr / 2;
                }
            } else {
                guess = guess + incr;
            }
        }

        self.recipe(Reactant::fuel(guess))
    }
}

#[derive(Debug)]
struct Recipe {
    reactions: Vec<Reaction>,
}

impl Recipe {
    fn new(reactions: Vec<Reaction>) -> Self {
        Recipe { reactions }
    }

    fn ore_requirement(&self) -> usize {
        self.reactions
            .iter()
            .flat_map(|r| {
                r.inputs
                    .iter()
                    .filter(|&r| r.chemical == Chemical::Ore)
                    .map(|r| r.quantity)
            })
            .sum()
    }

    fn outputs(&self) -> Vec<Reactant> {
        let mut supplies = CargoHold::new(self.ore_requirement());

        for reaction in &self.reactions {
            supplies.react(reaction).expect("Valid recipe!");
        }

        supplies.contents()
    }

    fn fuel_produced(&self) -> usize {
        self.outputs()
            .iter()
            .filter(|&r| r.chemical == Chemical::Fuel)
            .nth(0)
            .map(|r| r.quantity)
            .unwrap_or(0)
    }
}

#[derive(Debug, Default)]
struct RecipeBuilder {
    reactions: VecDeque<Reaction>,
    requirements: HashMap<Chemical, isize>,
}

impl RecipeBuilder {
    fn new(target: &Reactant) -> Self {
        let mut r = Self::default();
        r.requirements
            .insert(target.chemical.clone(), target.quantity as isize);
        r
    }

    fn is_finished(&self) -> bool {
        self.requirements
            .iter()
            .all(|(c, q)| c == &Chemical::Ore || *q <= 0)
    }

    fn chemical_requirement(&mut self) -> Option<&Chemical> {
        self.requirements
            .iter()
            .filter(|&(c, q)| c != &Chemical::Ore && *q > 0)
            .map(|(c, _)| c)
            .next()
    }

    fn ore_requirement(&self) -> usize {
        self.requirements
            .get(&Chemical::Ore)
            .map(|c| *c)
            .unwrap_or(0) as usize
    }

    fn add(&mut self, reaction: Reaction) -> Result<(), Error> {
        let output = reaction.output();
        let amount = self
            .requirements
            .get_mut(&output.chemical)
            .ok_or(anyhow!("Not yet producting {:?}", output.chemical))?;

        let mut n = amount.div_euclid(output.quantity as isize);
        if amount.rem_euclid(output.quantity as isize) != 0 {
            n += 1;
        }

        let reaction = reaction.repeat(n as usize);

        *amount -= reaction.output().quantity as isize;

        for input in &reaction.inputs {
            *self.requirements.entry(input.chemical.clone()).or_insert(0) +=
                input.quantity as isize;
        }
        self.reactions.push_front(reaction);

        Ok(())
    }

    fn build(self) -> Recipe {
        let mut supplies = CargoHold::new(self.ore_requirement() as usize);
        let mut results = Vec::with_capacity(self.reactions.len());

        let mut queue: VecDeque<_> = self.reactions;
        let mut attempts = 0;

        while let Some(reaction) = queue.pop_front() {
            if supplies.can(&reaction) {
                supplies
                    .react(&reaction)
                    .expect("Reaction should not fail!");
                results.push(reaction);
                attempts = 0;
            } else {
                attempts += 1;
                queue.push_back(reaction);
            }
            if attempts > queue.len() + 2 {
                panic!("Builder resulted in a recipe cycle!")
            }
        }

        Recipe::new(results)
    }
}

#[derive(Debug, Error)]
enum CargoHoldError {
    #[error("Not enough {0:?}")]
    InsufficientChemical(Chemical),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug)]
struct CargoHold {
    chemicals: HashMap<Chemical, usize>,
}

impl CargoHold {
    fn new(ore_available: usize) -> Self {
        let mut chemicals = HashMap::new();
        chemicals.insert(Chemical::Ore, ore_available);
        Self { chemicals }
    }

    fn has_reactant(&self, reactant: &Reactant) -> bool {
        self.chemicals
            .get(&reactant.chemical)
            .map(|q| *q >= reactant.quantity)
            .unwrap_or(false)
    }

    fn can(&self, reaction: &Reaction) -> bool {
        reaction.inputs.iter().all(|r| self.has_reactant(r))
    }

    fn react(&mut self, reaction: &Reaction) -> Result<(), CargoHoldError> {
        for input in &reaction.inputs {
            let source = self
                .chemicals
                .get_mut(&input.chemical)
                .ok_or(CargoHoldError::InsufficientChemical(input.chemical.clone()))?;
            *source = (*source)
                .checked_sub(input.quantity)
                .ok_or(CargoHoldError::InsufficientChemical(input.chemical.clone()))?;
        }
        *self
            .chemicals
            .entry(reaction.output.chemical.clone())
            .or_insert(0) += reaction.output.quantity;

        Ok(())
    }

    fn contents(&self) -> Vec<Reactant> {
        self.chemicals
            .iter()
            .filter(|&(_, q)| *q > 0)
            .map(|(c, q)| Reactant::new(c.clone(), *q))
            .collect()
    }
}

#[allow(dead_code, unused_variables)]
pub(crate) fn main(input: Box<dyn Read + 'static>) -> ::std::result::Result<(), Error> {
    let library = Library::read(input)?;

    let recipe = library.recipe(Reactant::new(Chemical::Fuel, 1))?;
    println!("Part 1: {} ORE required", recipe.ore_requirement());

    let trillion: usize = 1000000000000;
    let naive = trillion / recipe.ore_requirement();

    let recipe = library.consume(trillion)?;

    println!("Part 2: {} FUEL can be produced", recipe.fuel_produced());

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_chemical() {
        assert_eq!("ORE".parse::<Chemical>().unwrap(), Chemical::Ore);
        assert_eq!("FUEL".parse::<Chemical>().unwrap(), Chemical::Fuel);
        assert_eq!(
            "Fuel".parse::<Chemical>().unwrap(),
            Chemical::Named("Fuel".into())
        );
        assert_eq!(
            "A".parse::<Chemical>().unwrap(),
            Chemical::Named("A".into())
        );
    }

    #[test]
    fn parse_reactant() {
        assert_eq!(
            "10 ORE".parse::<Reactant>().unwrap(),
            Reactant::new(Chemical::Ore, 10)
        );
        assert_eq!(
            "7 A".parse::<Reactant>().unwrap(),
            Reactant::new(Chemical::Named("A".into()), 7)
        );
    }

    #[test]
    fn parse_reaction() {
        let reaction: Reaction = "10 ORE => 2 A".parse().unwrap();
        assert_eq!(reaction.inputs, vec![Reactant::new(Chemical::Ore, 10)]);
        let reaction: Reaction = "7 A, 1 E => 2 FUEL".parse().unwrap();
        assert_eq!(
            reaction.inputs,
            vec![
                Reactant::new(Chemical::Named("A".into()), 7),
                Reactant::new(Chemical::Named("E".into()), 1)
            ]
        );
        assert_eq!(reaction.output, Reactant::fuel(2));
    }

    #[test]
    fn example_a() {
        let library = Library::read(Box::new(
            "10 ORE => 10 A
        1 ORE => 1 B
        7 A, 1 B => 1 C
        7 A, 1 C => 1 D
        7 A, 1 D => 1 E
        7 A, 1 E => 1 FUEL"
                .as_bytes(),
        ))
        .unwrap();

        assert_eq!(
            library
                .recipe(Reactant::new(Chemical::Fuel, 1))
                .unwrap()
                .ore_requirement(),
            31
        );
    }

    #[test]
    fn example_b() {
        let library = Library::read(Box::new(
            "9 ORE => 2 A
            8 ORE => 3 B
            7 ORE => 5 C
            3 A, 4 B => 1 AB
            5 B, 7 C => 1 BC
            4 C, 1 A => 1 CA
            2 AB, 3 BC, 4 CA => 1 FUEL"
                .as_bytes(),
        ))
        .unwrap();

        let recipe = library.recipe(Reactant::new(Chemical::Fuel, 1)).unwrap();
        assert_eq!(recipe.ore_requirement(), 165);
    }

    #[test]
    fn example_c() {
        let library = Library::read(Box::new(
            "157 ORE => 5 NZVS
            165 ORE => 6 DCFZ
            44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
            12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
            179 ORE => 7 PSHF
            177 ORE => 5 HKGWZ
            7 DCFZ, 7 PSHF => 2 XJWVT
            165 ORE => 2 GPVTF
            3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT"
                .as_bytes(),
        ))
        .unwrap();

        assert_eq!(
            library
                .recipe(Reactant::new(Chemical::Fuel, 1))
                .unwrap()
                .ore_requirement(),
            13312
        );

        let recipe = library.consume(1000000000000).unwrap();

        assert_eq!(recipe.fuel_produced(), 82892753);
    }

    #[test]
    fn example_d() {
        let library = Library::read(Box::new(
            "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
            17 NVRVD, 3 JNWZP => 8 VPVL
            53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
            22 VJHF, 37 MNCFX => 5 FWMGM
            139 ORE => 4 NVRVD
            144 ORE => 7 JNWZP
            5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
            5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
            145 ORE => 6 MNCFX
            1 NVRVD => 8 CXFTF
            1 VJHF, 6 MNCFX => 4 RFSQX
            176 ORE => 6 VJHF"
                .as_bytes(),
        ))
        .unwrap();

        assert_eq!(
            library
                .recipe(Reactant::new(Chemical::Fuel, 1))
                .unwrap()
                .ore_requirement(),
            180697
        );

        let recipe = library.consume(1000000000000).unwrap();

        assert_eq!(recipe.fuel_produced(), 5586022);
    }

    #[test]
    fn example_e() {
        let library = Library::read(Box::new(
            "171 ORE => 8 CNZTR
            7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
            114 ORE => 4 BHXH
            14 VRPVC => 6 BMBT
            6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
            6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
            15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
            13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
            5 BMBT => 4 WPTQ
            189 ORE => 9 KTJDG
            1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
            12 VRPVC, 27 CNZTR => 2 XDBXC
            15 KTJDG, 12 BHXH => 5 XCVML
            3 BHXH, 2 VRPVC => 7 MZWV
            121 ORE => 7 VRPVC
            7 XCVML => 6 RJRHP
            5 BHXH, 4 VRPVC => 5 LTCX"
                .as_bytes(),
        ))
        .unwrap();

        assert_eq!(
            library
                .recipe(Reactant::new(Chemical::Fuel, 1))
                .unwrap()
                .ore_requirement(),
            2210736
        );

        let recipe = library.consume(1000000000000).unwrap();

        assert_eq!(recipe.fuel_produced(), 460664);
    }
}
