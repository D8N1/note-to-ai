# Note-to-AI Architecture & User Flow Diagrams

This document provides visual representations of the note-to-ai system architecture and user workflows to help users understand how the system operates.

## 🏗️ System Architecture Diagram

```mermaid
graph TB
    %% User Interfaces
    User[👤 User]
    Signal[📱 Signal App]
    CLI[💻 CLI Interface]
    Obsidian[📝 Obsidian Vault]
    
    %% Core System
    NoteToAI[🧠 note-to-ai Core]
    
    %% Processing Layers
    VoiceProc[🎤 Voice Processing<br/>Whisper ASR]
    TextProc[📝 Text Processing<br/>NLP Pipeline]
    AIEngine[🤖 AI Engine<br/>Local LLM]
    
    %% Storage & Indexing
    VaultIdx[📚 Vault Indexer<br/>BLAKE3 + SQLite]
    ObsidianInt[🔗 Obsidian Integration<br/>Markdown Generator]
    SearchEng[🔍 Search Engine<br/>Text + Semantic]
    
    %% Data Storage
    SQLiteDB[(🗄️ SQLite Database<br/>Metadata + Index)]
    VaultFiles[(📁 Vault Files<br/>Markdown Documents)]
    ModelStore[(🧠 Model Storage<br/>Local AI Models)]
    
    %% External Services
    Crypto[🔐 Crypto Layer<br/>Quantum-Resistant]
    P2PNet[🌐 P2P Network<br/>IPFS + libp2p]
    
    %% User Inputs
    User -->|Voice Messages| Signal
    User -->|Direct Commands| CLI
    User -->|View/Edit Notes| Obsidian
    
    %% Signal Processing Flow
    Signal -->|Encrypted Messages| NoteToAI
    NoteToAI --> VoiceProc
    VoiceProc -->|Transcribed Text| TextProc
    
    %% CLI Processing Flow
    CLI --> NoteToAI
    NoteToAI --> TextProc
    
    %% Core Processing
    TextProc --> AIEngine
    TextProc --> VaultIdx
    TextProc --> SearchEng
    
    %% AI Processing
    AIEngine --> ModelStore
    AIEngine -->|Generated Response| ObsidianInt
    
    %% Storage Operations
    VaultIdx --> SQLiteDB
    VaultIdx --> VaultFiles
    SearchEng --> SQLiteDB
    SearchEng --> VaultFiles
    
    %% Output Generation
    ObsidianInt --> VaultFiles
    ObsidianInt -->|Formatted Notes| Obsidian
    NoteToAI -->|Responses| Signal
    NoteToAI -->|Status/Results| CLI
    
    %% Security & Networking
    NoteToAI --> Crypto
    NoteToAI --> P2PNet
    Crypto --> P2PNet
    
    %% Styling
    classDef userInterface fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef coreSystem fill:#f3e5f5,stroke:#4a148c,stroke-width:3px
    classDef processing fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef storage fill:#fff3e0,stroke:#e65100,stroke-width:2px
    classDef security fill:#ffebee,stroke:#b71c1c,stroke-width:2px
    
    class User,Signal,CLI,Obsidian userInterface
    class NoteToAI coreSystem
    class VoiceProc,TextProc,AIEngine,VaultIdx,ObsidianInt,SearchEng processing
    class SQLiteDB,VaultFiles,ModelStore storage
    class Crypto,P2PNet security
```

## 🔄 User Workflow Diagram

```mermaid
flowchart TD
    Start([🚀 User Starts])
    
    %% Input Methods
    ChooseInput{Choose Input Method}
    VoiceInput[🎤 Send Voice Note<br/>to Signal]
    TextInput[💬 Send Text Message<br/>to Signal]
    CLIInput[💻 Use CLI Command]
    DirectEdit[✏️ Edit Obsidian<br/>Directly]
    
    %% Processing Steps
    VoiceToText[🔄 Voice → Text<br/>Whisper ASR]
    ParseQuery[🧩 Parse & Understand<br/>Query Intent]
    SearchKnowledge[🔍 Search Knowledge Base<br/>Semantic + Text Search]
    GenerateAI[🤖 Generate AI Response<br/>Local LLM + RAG]
    
    %% Output Formatting
    FormatResponse[📋 Format Response<br/>Obsidian Markdown]
    SaveToVault[💾 Save to Vault<br/>Organized Structure]
    
    %% User Receives Output
    SignalReply[📱 Receive Signal Reply<br/>with Summary]
    ObsidianNote[📝 View in Obsidian<br/>Full Formatted Note]
    CLIOutput[💻 CLI Status/Results]
    
    %% Continuous Learning
    IndexNewContent[📚 Index New Content<br/>for Future Searches]
    UpdateKnowledge[🧠 Update Knowledge Graph<br/>Links & Relationships]
    
    End([✅ Complete])
    
    %% Flow Connections
    Start --> ChooseInput
    
    %% Input Paths
    ChooseInput -->|Voice Message| VoiceInput
    ChooseInput -->|Text Message| TextInput
    ChooseInput -->|Command Line| CLIInput
    ChooseInput -->|Direct Edit| DirectEdit
    
    %% Voice Processing Path
    VoiceInput --> VoiceToText
    VoiceToText --> ParseQuery
    
    %% Text Processing Path
    TextInput --> ParseQuery
    CLIInput --> ParseQuery
    
    %% Core Processing
    ParseQuery --> SearchKnowledge
    SearchKnowledge --> GenerateAI
    GenerateAI --> FormatResponse
    FormatResponse --> SaveToVault
    
    %% Output Distribution
    SaveToVault --> SignalReply
    SaveToVault --> ObsidianNote
    SaveToVault --> CLIOutput
    
    %% Continuous Learning
    SaveToVault --> IndexNewContent
    IndexNewContent --> UpdateKnowledge
    UpdateKnowledge --> End
    
    %% Direct Edit Path
    DirectEdit --> IndexNewContent
    
    %% Return Paths
    SignalReply --> End
    ObsidianNote --> End
    CLIOutput --> End
    
    %% Styling
    classDef startEnd fill:#c8e6c9,stroke:#2e7d32,stroke-width:3px
    classDef input fill:#e3f2fd,stroke:#1565c0,stroke-width:2px
    classDef process fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef output fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    classDef learning fill:#ffebee,stroke:#c62828,stroke-width:2px
    
    class Start,End startEnd
    class VoiceInput,TextInput,CLIInput,DirectEdit input
    class VoiceToText,ParseQuery,SearchKnowledge,GenerateAI,FormatResponse,SaveToVault process
    class SignalReply,ObsidianNote,CLIOutput output
    class IndexNewContent,UpdateKnowledge learning
```

## 🎯 Key User Scenarios

### Scenario 1: Voice-to-Knowledge Workflow
```
📱 User sends voice note via Signal
    ↓
🎤 Whisper transcribes audio to text
    ↓
🧩 System parses intent and extracts key concepts
    ↓
🔍 Searches existing knowledge base for relevant context
    ↓
🤖 Local LLM generates comprehensive response with RAG
    ↓
📝 Creates formatted Obsidian note with metadata, tags, links
    ↓
💾 Saves to organized vault structure (/AI Responses/YYYY-MM-DD/)
    ↓
📱 Sends summary back to Signal + 📝 Full note available in Obsidian
```

### Scenario 2: Research & Knowledge Building
```
💻 User runs CLI indexing command
    ↓
📚 System scans vault for new/changed markdown files
    ↓
🔐 BLAKE3 hashing detects content changes
    ↓
🗄️ Updates SQLite database with metadata and full-text index
    ↓
🔍 Builds searchable knowledge graph
    ↓
🤖 Future queries can leverage this indexed knowledge
    ↓
🧠 Continuous learning and knowledge accumulation
```

### Scenario 3: Daily Knowledge Management
```
🌅 Daily Note Creation:
   - Automatic daily note generation
   - Time-stamped interaction logs
   - Cross-linked to relevant knowledge

📝 AI Response Notes:
   - Query-based file naming
   - Structured frontmatter with metadata
   - Automatic tag generation and linking

🔗 Knowledge Graph Building:
   - Automatic [[wikilink]] generation
   - Tag-based organization (#ai-generated, #research)
   - Temporal knowledge tracking
```

## 🛠️ System Components Detail

### Core Technologies Stack
```
Frontend Interfaces:
├── 📱 Signal (Encrypted Messaging)
├── 💻 CLI (Command Line Interface)  
└── 📝 Obsidian (Knowledge Management)

Processing Engine:
├── 🎤 Whisper (Speech-to-Text)
├── 🧠 Local LLM (Hermes-3-8B)
├── 🔍 Hybrid Search (Text + Semantic)
└── 📚 Document Indexer (BLAKE3 + SQLite)

Storage Layer:
├── 🗄️ SQLite Database (Metadata & Index)
├── 📁 Markdown Files (Human-readable knowledge)
└── 🧠 Local AI Models (Self-contained inference)

Security & Privacy:
├── 🔐 End-to-end encryption (Signal Protocol)
├── 🛡️ Quantum-resistant cryptography
├── 🏠 Local-first architecture (no cloud dependency)
└── 🌐 Optional P2P synchronization (IPFS)
```

### Data Flow Architecture
```
Input → Processing → Storage → Output
  ↓         ↓          ↓        ↓
Voice     Parse     SQLite   Signal
Text   → Search  → Markdown → Obsidian
CLI      AI Gen    Models     CLI
```

## 🎯 Benefits Summary

### For Users:
- **🏠 Privacy-First**: All processing happens locally
- **🧠 Intelligent**: AI-powered responses with context from your knowledge
- **📝 Organized**: Automatic knowledge management in Obsidian format
- **🔍 Searchable**: Full-text and semantic search across all content
- **⚡ Fast**: Local processing, no network dependencies for core features

### For Developers:
- **🦀 Rust Performance**: High-performance, memory-safe implementation
- **🔧 Modular Design**: Clean separation of concerns
- **🧪 Testable**: Comprehensive test suite with benchmarks
- **📦 Self-contained**: Minimal external dependencies
- **🔒 Secure**: Quantum-resistant cryptography foundation

---

*This architecture enables a completely local-first AI knowledge management system that respects user privacy while providing powerful AI-assisted note-taking and knowledge discovery capabilities.*
