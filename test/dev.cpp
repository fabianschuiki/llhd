/* Copyright (c) 2014 Fabian Schuiki */
#include <fstream>
#include <iostream>
#include <set>
#include <stack>
#include <vector>

struct Token
{
	Token *prev;
	Token() : prev(NULL) {}
};

struct ByteToken : public Token
{
	char c;
};

struct LiteralToken : public Token
{
	std::string text;
	explicit LiteralToken(const std::string &text) : Token(), text(text) {}
	explicit LiteralToken(char c) : Token(), text(c, 1) {}
};

struct EndOfInputToken : public Token
{
};

enum MatchResult {
	kDiscard, // rule does not apply
	kReduce,  // rule applies to all tokens in the lead
	kNeedMore // rule requires more tokens to decide
};

struct Rule;
struct RuleIterator
{
	Rule *r;
	int i;
	explicit RuleIterator(Rule *r, int i) : r(r), i(i) {}
};

typedef std::vector<Token *> TokenVector;

struct RuleLead
{
	RuleLead *parent;
	Rule *rule;
	Token *firstToken;
	Token *lastToken;
	int i;

	explicit RuleLead(Rule *rule) : parent(NULL), firstToken(NULL), lastToken(NULL), rule(rule), i(0) {}
	~RuleLead() {
		Token *t = lastToken;
		while (t) {
			Token *tn = t->prev;
			delete t;
			t = tn;
		}
	}
};

// struct RuleLeadSet
// {
// 	std::vector<RuleLead> store;
// 	std::set<RuleLead *> active;

// 	void insert(const RuleLead &rl) {
// 		store.push_back(rl).first;
// 		active.insert(&*i);
// 	}

// 	void erase(RuleLead *rl) {

// 	}

// 	void activate(RuleLead *rl) { active.insert(rl); }
// 	void deactivate(RuleLead *rl) { active.erase(rl); }
// };

typedef std::set<RuleLead *> RuleLeadSet;


struct Rule
{
	std::string name;
	virtual MatchResult match(const ByteToken &t, RuleLead *&lead, RuleLeadSet& leads) = 0;
};

// struct SequenceRule : public Rule { std::vector<Rule *> sequence; };
// struct OrRule : public Rule { std::vector<Rule *> options; };

struct RepeatRule : public Rule
{
	Rule *rule;

	explicit RepeatRule(Rule *rule) : rule(rule) {}

	virtual MatchResult match(const ByteToken &t, RuleLead *&lead, RuleLeadSet &leads)
	{
		RuleLead *sublead = new RuleLead(rule);
		sublead->parent = lead;
		MatchResult mr = rule->match(t, sublead, leads);

		switch (mr) {
		case kNeedMore:
			return kNeedMore;
		case kReduce:
			leads.insert(lead);
			sublead->firstToken->prev = lead->lastToken;
			lead->lastToken = sublead->lastToken;
			if (lead->i == 0) lead->firstToken = sublead->firstToken;
			sublead->lastToken = NULL;
			lead->i++;
			delete sublead;
			return kNeedMore;
		case kDiscard:
			delete sublead;
			if (lead->i > 0) {
				std::cout << "reducing RepeatRule with " << lead->i << " tokens\n";
				return kReduce;
			} else {
				return kDiscard;
			}
		}
	}
};

struct OrRule : public Rule
{
	std::vector<Rule *> rules;

	virtual MatchResult match(const ByteToken &t, RuleLead *&lead, RuleLeadSet &leads)
	{
		MatchResult mr = kDiscard;
		for (int i = 0; i < rules.size(); i++) {
			RuleLead *sublead = new RuleLead(rule);
			sublead->parent = lead;
			MatchResult mr = rule->match(t, sublead, leads);

			switch (mr) {
			case kNeedMore:
				mr = kNeedMore;
			case kReduce:
				leads.insert(lead);
				sublead->firstToken->prev = lead->lastToken;
				lead->lastToken = sublead->lastToken;
				if (lead->i == 0) lead->firstToken = sublead->firstToken;
				sublead->lastToken = NULL;
				lead->i++;
				delete sublead;
				return kNeedMore;
			case kDiscard:
				delete sublead;
				if (lead->i > 0) {
					std::cout << "reducing RepeatRule with " << lead->i << " tokens\n";
					return kReduce;
				} else {
					return kDiscard;
				}
			}
		}
	}
};

struct CharacterSetRule : public Rule
{
	std::set<char> chars;

	explicit CharacterSetRule(const char *s)
	{
		while (*s != 0) chars.insert(*s++);
	}

	virtual MatchResult match(const ByteToken &t, RuleLead *&lead, RuleLeadSet& leads)
	{
		if (chars.count(t.c)) {
			lead->firstToken = lead->lastToken = new LiteralToken(t.c);
			return kReduce;
		} else {
			return kDiscard;
		}
	}
};

int main(int argc, char **argv)
{
	if (argc != 2) {
		std::cerr << "usage: " << argv[0] << " <filename>\n";
		return 1;
	}
	std::ifstream fin(argv[1]);

	// Define some rules to be used.
	CharacterSetRule r0("&'()*+,-./:;<=>`|[]?@"); r0.name = "symbol";
	CharacterSetRule r1(" \t\r\n"); r0.name = "whitespace";
	RepeatRule r2(&r0); r0.name = "symbols";

	RuleLeadSet leads;
	leads.insert(new RuleLead(&r2));

	// Perform the raw byte-level pass.
	ByteToken bt; int guard = 10;
	while (fin.good() && guard > 0) {
		bt.c = fin.get();
		if (!fin.good())
			break;

		std::cout << "[leads: " << leads.size() << "] character " << bt.c << '\n';
		RuleLeadSet newLeads;
		for (RuleLeadSet::iterator i = leads.begin(); i != leads.end(); i++) {
			RuleLead *lead = *i;
			while (lead) {
				MatchResult mr = lead->rule->match(bt, lead, newLeads);
				if (mr == kReduce) {
					std::cout << "found solution with " << lead->i << " tokens\n";
					delete lead;
					lead = NULL;
				} else if (mr == kDiscard) {
					std::cout << "discarding lead\n";
					RuleLead *nl = lead->parent;
					delete lead;
					lead = nl;
				} else {
					lead = NULL;
				}
			}
		}
		leads = newLeads;

		if (leads.empty()) {
			std::cout << "illegal character " << bt.c << '\n';
			guard--;
		}
	}

	return 0;
}