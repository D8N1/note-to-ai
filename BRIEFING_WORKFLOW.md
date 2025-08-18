# Executive Briefing Workflow

## Daily Intelligence Pipeline

```ascii
                    ┌─────────────────────────────────────────────────────────────┐
                    │                    OVERNIGHT PROCESSING                     │
                    │                       (2300-0600)                          │
                    └─────────────────────────────────────────────────────────────┘
                                                    │
                    ┌─────────────────────────────────────────────────────────────┐
                    │  📊 KNOWLEDGE AGGREGATION ENGINE                           │
                    │  ├── Analyze last 24h Signal "Note to Self" messages       │
                    │  ├── Parse voice transcripts, images, documents            │
                    │  ├── Extract action items, insights, connections           │
                    │  ├── Cross-reference with existing knowledge base          │
                    │  └── Identify trends and strategic implications            │
                    └─────────────────────────────────────────────────────────────┘
                                                    │
                    ┌─────────────────────────────────────────────────────────────┐
                    │  🧠 AI BRIEFING GENERATION                                 │
                    │  ├── Hermes 3 8B: Strategic analysis & synthesis           │
                    │  ├── DistilBART: Executive summary generation               │
                    │  ├── T5-Small: Question formulation for follow-up          │
                    │  └── Context preparation for external LLM optimization     │
                    └─────────────────────────────────────────────────────────────┘
                                                    │
                    ┌─────────────────────────────────────────────────────────────┐
                    │  📝 DOCUMENT GENERATION (0600 DELIVERY)                   │
                    │                                                             │
                    │  ┌─────────────────────┐    ┌─────────────────────────────┐ │
                    │  │   FULL BRIEF.md     │    │   EXEC_SUMMARY.md           │ │
                    │  │                     │    │                             │ │
                    │  │ • Strategic Overview│    │ • 3-min read highlights     │ │
                    │  │ • Action Items      │    │ • Critical decisions        │ │
                    │  │ • Key Insights      │    │ • Priority actions          │ │
                    │  │ • Trend Analysis    │    │ • Risk factors              │ │
                    │  │ • Dependencies      │    │ • Opportunities             │ │
                    │  │ • Research Prompts  │    │ • Today's focus areas       │ │
                    │  └─────────────────────┘    └─────────────────────────────┘ │
                    └─────────────────────────────────────────────────────────────┘
                                                    │
                    ┌─────────────────────────────────────────────────────────────┐
                    │  🎤 AUDIO BRIEFING GENERATION                              │
                    │                                                             │
                    │  ┌─────────────────────┐    ┌─────────────────────────────┐ │
                    │  │  FULL_BRIEF.mp3     │    │  EXEC_SUMMARY.mp3           │ │
                    │  │                     │    │                             │ │
                    │  │ • 15-20 min format  │    │ • 3-5 min format            │ │
                    │  │ • Professional tone │    │ • Urgent items only         │ │
                    │  │ • Detailed analysis │    │ • Decision-ready format     │ │
                    │  │ • "Executive Asst"  │    │ • "Morning Brief" style     │ │
                    │  │   persona delivery  │    │   persona delivery          │ │
                    │  └─────────────────────┘    └─────────────────────────────┘ │
                    └─────────────────────────────────────────────────────────────┘
                                                    │
                    ┌─────────────────────────────────────────────────────────────┐
                    │  📱 SIGNAL DELIVERY (0600 SHARP)                          │
                    │                                                             │
                    │  Signal "Note to Self" receives:                           │
                    │  ├── 📄 FULL_BRIEF.md (attachment)                        │
                    │  ├── 📄 EXEC_SUMMARY.md (attachment)                      │
                    │  ├── 🎵 FULL_BRIEF.mp3 (audio message)                   │
                    │  ├── 🎵 EXEC_SUMMARY.mp3 (audio message)                 │
                    │  └── 💬 "Good morning. Your briefing is ready."           │
                    └─────────────────────────────────────────────────────────────┘
                                                    │
                    ┌─────────────────────────────────────────────────────────────┐
                    │                   INTERACTIVE COMMANDS                      │
                    │                     (Throughout Day)                       │
                    └─────────────────────────────────────────────────────────────┘
                                                    │
    ┌───────────────────────┬─────────────────────┼─────────────────────┬───────────────────────┐
    │                       │                     │                     │                       │
┌───▼───┐               ┌───▼───┐             ┌───▼───┐             ┌───▼───┐               ┌───▼───┐
│ /drill│               │ /focus│             │ /prep │             │ /author│              │ /prompt│
│       │               │       │             │       │             │        │              │        │
│Extract│               │Deep   │             │Action │             │Call    │              │Generate│
│section│               │dive   │             │item   │             │author  │              │external│
│with   │               │into   │             │prep   │             │of      │              │LLM     │
│context│               │topic  │             │mode   │             │section │              │prompts │
└───┬───┘               └───┬───┘             └───┬───┘             └───┬────┘              └───┬───┘
    │                       │                     │                     │                       │
    ▼                       ▼                     ▼                     ▼                       ▼
```

## Command Specifications

### /drill [section_id]
```
Input:  "/drill supply_chain"
Output: 
├── 📄 Detailed supply chain analysis from brief
├── 🎤 Audio explanation (2-3 min)
├── 📊 Related context from knowledge base
├── ❓ Follow-up questions for deeper analysis
└── 🔗 Connected insights from previous briefs
```

### /focus [topic]
```
Input:  "/focus Q1_revenue"
Output:
├── 📄 All Q1 revenue mentions across recent briefs
├── 📈 Trend analysis and projections
├── 🎯 Action items and dependencies
├── ⚠️  Risk factors and mitigation strategies
└── 💡 Strategic recommendations
```

### /prep [action_item_id]
```
Input:  "/prep board_presentation"
Output:
├── 📋 Action item breakdown and timeline
├── 📚 Supporting materials from knowledge base
├── 🎯 Key talking points and data
├── ❓ Anticipated questions and answers
└── 📝 Prep checklist with deadlines
```

### /author [section_id]
```
Input:  "/author supply_chain"
Output:
├── 👤 "Sarah Martinez, Supply Chain Director"
├── 🎤 Audio briefing in Sarah's persona/expertise
├── 📞 Simulated Q&A session with Sarah
├── 📈 Sarah's historical insights and patterns
└── 🤝 Recommended actions based on Sarah's style
```

### /prompt [research_topic]
```
Input:  "/prompt competitor_analysis"
Output:
├── 🎯 Optimized prompts for external LLMs
├── 📊 Context package for Claude/GPT-4
├── 🔍 Research questions prioritized by urgency
├── 💰 Token-optimized queries to minimize API costs
└── 📋 Expected deliverables and success metrics
```

## Technical Architecture

```ascii
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              BRIEFING SCHEDULER                                     │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐                │
│  │   CRON ENGINE   │    │  BRIEF BUILDER  │    │  AUDIO ENGINE   │                │
│  │                 │    │                 │    │                 │                │
│  │ • 0600 trigger  │───▶│ • MD generation │───▶│ • TTS synthesis │                │
│  │ • Daily cycle   │    │ • Template sys  │    │ • Voice persona │                │
│  │ • Retry logic   │    │ • Data fusion   │    │ • Audio quality │                │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘                │
│                                                                                     │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                            COMMAND PROCESSOR                                        │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐                │
│  │  PARSER ENGINE  │    │  CONTEXT ENGINE │    │ RESPONSE ENGINE │                │
│  │                 │    │                 │    │                 │                │
│  │ • /command regex│───▶│ • Section lookup│───▶│ • Multi-format  │                │
│  │ • Parameter ext │    │ • Author mapping│    │ • Audio/Text    │                │
│  │ • Validation    │    │ • Knowledge RAG │    │ • Persona voice │                │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘                │
│                                                                                     │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                            SIGNAL INTEGRATION                                       │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐                │
│  │ MESSAGE MONITOR │    │  FILE SENDER    │    │  AUDIO SENDER   │                │
│  │                 │    │                 │    │                 │                │
│  │ • Watch inbox   │───▶│ • MD attachments│    │ • MP3 messages  │                │
│  │ • Parse commands│    │ • Inline preview│    │ • Voice messages│                │
│  │ • Route to proc │    │ • Metadata tags │    │ • Streaming opt │                │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘                │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## File Structure

```
briefings/
├── templates/
│   ├── full_brief.md.template
│   ├── exec_summary.md.template
│   ├── audio_script_full.template
│   └── audio_script_summary.template
├── personas/
│   ├── executive_assistant.yaml
│   ├── department_heads/
│   │   ├── sarah_supply_chain.yaml
│   │   ├── mike_finance.yaml
│   │   └── alex_strategy.yaml
├── commands/
│   ├── drill.rs
│   ├── focus.rs
│   ├── prep.rs
│   ├── author.rs
│   └── prompt.rs
└── scheduler/
    ├── cron_config.toml
    ├── briefing_engine.rs
    └── audio_synthesis.rs
```

## Sample Command Flows

### Morning Routine (0600)
```
[AUTOMATED]
1. Generate FULL_BRIEF.md + EXEC_SUMMARY.md
2. Synthesize audio versions with Executive Assistant persona
3. Send all 4 files to Signal "Note to Self"
4. Log briefing metrics and knowledge base updates
```

### Mid-Day Deep Dive (1200)
```
User: "/drill supply_chain"
System: 
├── Extract supply chain section from morning brief
├── Pull related context from last 7 days
├── Generate audio explanation with supply chain expert persona
├── Suggest 3 follow-up actions
└── Prepare external LLM prompts for deeper research
```

### Evening Prep (1800)
```
User: "/prep board_presentation"
System:
├── Compile all board-related mentions from briefs
├── Generate talking points and supporting data
├── Create Q&A scenarios with anticipated questions
├── Package materials for tomorrow's review
└── Set reminder for final prep at 0800
```
