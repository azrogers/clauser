need to support the following situations:

Where the fields are mostly known (for example, state definitions):
```
class State : ClObject {
	int id;
	string subsistence_building;
	vector<string> provinces;
	vector<string> traits;
	optional<string> city;
	optional<string> farm;
	optional<string> mine;
	optional<string> wood;
	int arable_land;
	vector<string> arable_resources;
	// SomeOf representing some of a set of known keys with defaults for missing
	SomeOf<CappedResources> capped_resources;
	optional<Resource> resource;
	optional<int> naval_exit_id;
}
```

Where the fields might have any number of conditions:
```
class Decision : ClObject {
	EvaluatesTo<bool> is_shown;
	EvaluatesTo<bool> possible;
	ActionList when_taken;
	EvaluatesTo<int> ai_chance = 1;
}
```