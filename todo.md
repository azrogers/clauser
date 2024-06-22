### todo: 

- serialization
- better tests
- scripting runtime
  - separate from serde
  - scopes and directives
    - proper traits so that different projects can define their own set of directives and scopes
  - how efficient/insane can it get?
    - level 1: allow deserializing blocks and values as futures that take and mutate scope
    - level 2: compile directives into bytecode and only store and execute the bytecode (aim for this)
    - level 3: compile-time compilation of clausewitz into rust?

#### scripting runtime api

it might look something like this:
```
struct CountryScope {
  #[required]
  capital: usize,
  oob: String,
  technology: Map<String, bool>,
  ideas: Vec<String>,
  characters: Vec<String>
}

type DirectiveResult = Result<Option<Vec<ExecutionUnit>>, Error>;

// these would be compiled by macro into Directive structs

#[directive(CountryScope)]
fn set_technology(scope: &mut CountryScope, techs: HashMap<String, bool>) -> DirectiveResult {
  for (k, v) in techs {
    scope.technology.insert(k, v);
  }

  Ok(None)
}

#[directive(CountryScope)]
fn recruit_character(scope: &mut CountryScope, name: &str) -> DirectiveResult {
  scope.characters.push(name);
  Ok(None)
}

// allows access to the list of directives created by the macro through the CountryDirectives struct
directives!(CountryScope, CountryDirectives);

// create a derived scope CountryHistoryDirectives that looks first to CountryDirectives
// and then to GlobalDirectives. scopes are nested and parent scopes can be affected.
// runtime will need to maintain a stack of current scopes to apply all directives
directives_union!(GlobalDirectives, CountryDirectives, CountryHistoryDirectives);

struct CountryHistoryReader {}

impl ScriptReader for CountryHistoryReader {
  // top level scope of the document
  type Scope = CountryScope;
  // the applicable directives to reading this script
  type Directives = CountryHistoryDirectives;

  fn read(reader: &Reader) -> Result<Vec<ExecutionUnit>, Error> {
    // turn blocks of directives into execution units based on what can run together.
    // triggers get turned into their own execution units with their dependencies (trigger conditions) listed in the execution unit.
    // execution units can return other execution units, until the whole thing is read.
    // also, need some way of calling out to serde to do normal deserialization tasks at the same time.
    // this will probably involve macros too.
    // draw the rest of the owl.
  }
}
```

at the end of the reading process we should have a set of execution units, some linked to actions and triggers. later on we can have implementers of the execution unit trait have to generate bytecode from them.

caller needs some way of specifying that things like, object properties with a name like "1910.1.1" are triggers that occur on that date. don't want to hardcore because we don't know all the requirements. maybe an advanced directive trait like:
```
#[directive_re = "(\d{4})\.(\d{1,2}).(\d{1,2})"]
fn year_trigger(scope: &CountryScope, matches: &Matches) -> DirectiveResult {
  Ok(Some(vec![ ExecutionUnit::trigger(year_condition(matches), /* idk somehow call the deserializer to get child ExecutionUnits here */)]))
}
```