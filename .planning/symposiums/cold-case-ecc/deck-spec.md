# Slide Deck Specification: Cold Case Investigation via ECC

**Instructions for slide generation**: Create a professional Google Slides presentation with a dark theme (navy/charcoal backgrounds, white text, accent colors: emerald green for positive/solutions, amber for data points, red for problems). Use clean sans-serif fonts. Each slide should be visually driven with minimal text — use diagrams, charts, and icons where specified. The deck should feel like a premium technology pitch to law enforcement leadership and academics.

---

## Slide 1: Title

**Title**: Solving Cold Cases with Computational Causal Analysis
**Subtitle**: Adapting AI-Powered System Intelligence for Homicide Investigation
**Footer**: WeaveLogic | Confidential Draft
**Visual**: Abstract network graph in emerald on dark background (subtle, not distracting)

---

## Slide 2: The Scale of the Problem

**Title**: 250,000+ Unsolved Homicide Cases in the United States

**Key stats** (large numbers, one per quadrant):
- **250,000+** unsolved homicide cases
- **61.4%** national clearance rate (2024)
- **~40%** of homicides go unsolved every year
- **Germany: 90%+** clearance rate — proving it's solvable

**Bottom note**: Sources: FBI UCR, Bureau of Justice Statistics, DOJ

---

## Slide 3: Why Cases Go Cold

**Title**: Five Categories of Case Failure

**Visual**: Five icons in a row, each with a label and one-line description:

1. **Witness Attrition** — Witnesses move, forget, die, become uncooperative
2. **Evidence Degradation** — DNA degrades, tapes deteriorate, chain of custody breaks
3. **Investigator Turnover** — Lead detective retires, institutional knowledge lost
4. **Resource Constraints** — New cases always take priority over old ones
5. **Analytical Gaps** — Questions that were never asked, evidence that was never tested

**Callout box**: "The biggest factor isn't missing evidence — it's missing questions."

---

## Slide 4: Current Tools Fall Short

**Title**: Existing Technology Doesn't Solve the Core Problem

**Two-column layout**:

**Left column — What Exists**:
- IBM i2 Analyst's Notebook — link analysis, manual
- Palantir Gotham — data integration, expensive ($$$)
- VICAP — cross-case matching, but DOJ audit found limited participation
- Genetic genealogy — powerful but only works for DNA cases
- CODIS/AFIS/NIBIN — database lookups, not analysis

**Right column — What's Missing**:
- No tool identifies what questions weren't asked
- No tool measures case completeness
- No tool tests competing hypotheses against evidence
- No tool continuously re-evaluates as new information arrives
- No tool surfaces what evidence should be tested next

---

## Slide 5: Introducing ECC — Embodied Causal Cognition

**Title**: A System That Thinks Like a Detective

**Visual**: Diagram showing the DEMOCRITUS loop as a circular process:

```
    SENSE → EMBED → SEARCH → UPDATE → CROSSREF → PRUNE → REPORT
      ↑                                                      │
      └──────────────────────────────────────────────────────┘
                     Continuous Re-evaluation
```

**Three key capabilities** (icons + one line each):
- **Causal Graphs** — Map relationships between evidence, people, events, locations
- **Gap Analysis** — Identify what's MISSING from the investigation
- **Coherence Scoring** — Quantify how well the evidence fits together

**Footer**: Originally built for analyzing complex software systems. Same methodology applies to complex investigations.

---

## Slide 6: ECC Demo — What It Looks Like Today

**Title**: Live System Analysis (Software Domain)

**Visual**: Screenshot of the WeftOS knowledge graph visualization showing:
- Nodes (colored by type/community)
- Edges (colored by relationship type)
- Coherence gauge
- Search/filter interface

**Callout**: "Every node is a piece of evidence. Every edge is a relationship. The system finds the gaps."

**Note to presenter**: This is where you do a live demo of the browser sandbox at weftos.weavelogic.ai/clawft/ showing the knowledge graph, ExoChain log, and governance panel.

---

## Slide 7: The Case Graph Model

**Title**: Representing a Homicide Investigation as a Knowledge Graph

**Visual**: A sample graph diagram showing interconnected nodes:

**Node types** (color-coded circles):
- Red: PERSON (victim, suspect, witness)
- Blue: EVIDENCE (physical, testimonial, digital)
- Yellow: EVENT (crime, interview, arrest)
- Green: LOCATION (crime scene, addresses)
- Purple: TIMELINE (dates with uncertainty ranges)
- Gray: HYPOTHESIS (investigator theories)

**Edge types** (labeled arrows between nodes):
- WITNESSED_BY, FOUND_AT, CONTRADICTS, CORROBORATES, ALIBIED_BY, IDENTIFIED_BY

Show a mini example: Victim → FOUND_AT → Location → WITNESSED_BY → Witness → CONTRADICTS → Suspect Alibi

---

## Slide 8: Gap Analysis — The Killer Feature

**Title**: The System Tells You What You Don't Know

**Visual**: Same graph as Slide 7 but with RED DASHED CIRCLES around missing nodes:

**Example gaps surfaced**:
| Gap Found | Type | Priority |
|-----------|------|----------|
| Witness mentioned in report but never interviewed | Uninterviewed witness | HIGH |
| DNA collected from scene but never submitted to CODIS | Untested evidence | CRITICAL |
| Suspect claimed alibi at restaurant — never verified | Unverified alibi | HIGH |
| Cell tower records for 3-hour window never subpoenaed | Missing records | MEDIUM |
| Similar MO case in adjacent jurisdiction never cross-referenced | Unlinked case | MEDIUM |

**Callout box** (emerald accent): "Coherence score: 0.42 → Testing evidence item #47 would raise coherence to 0.71"

---

## Slide 9: Coherence Scoring — Quantifying Case Completeness

**Title**: A Mathematical Measure of How Well Evidence Connects

**Visual**: Two side-by-side graph diagrams:

**Left — Weak Case (Coherence: 0.23)**
- Disconnected clusters of evidence
- Many isolated nodes
- Red highlighting on gaps
- Caption: "Evidence doesn't connect. Multiple unresolved questions."

**Right — Strong Case (Coherence: 0.87)**
- Well-connected graph
- Strong cross-references
- Green highlighting
- Caption: "Evidence corroborates across multiple independent sources."

**Bottom**: "Coherence score uses spectral analysis (algebraic connectivity) — the same math used to measure network resilience. A case with coherence > 0.7 has strong evidentiary connections."

---

## Slide 10: Crime Scene Reconstruction

**Title**: Testing Hypotheses Against the Evidence Graph

**Visual**: Two competing timeline diagrams, side by side:

**Hypothesis A** (Prosecution theory):
- Timeline: 8:00 PM → 9:15 PM → 10:30 PM → 11:45 PM
- Events aligned with evidence items
- Coherence: 0.72
- Conflicts: 1

**Hypothesis B** (Alternative suspect):
- Timeline: 7:30 PM → 9:00 PM → 10:00 PM → 12:15 AM
- Different event sequence
- Coherence: 0.58
- Conflicts: 3

**Callout**: "The system doesn't decide guilt — it measures which theory best fits ALL the evidence, including exculpatory evidence."

---

## Slide 11: Data Sources — What Feeds the Graph

**Title**: Building the Case Graph from Existing Records

**Visual**: Hub-and-spoke diagram with "Case Graph" in center and data sources radiating out:

**Primary sources** (inner ring):
- Police reports (CAD/RMS)
- Witness statements & transcripts
- Forensic reports (autopsy, DNA, ballistics, prints, tox)
- Physical evidence logs & chain of custody

**Secondary sources** (outer ring):
- Cell tower records
- Surveillance footage logs
- Social media archives
- Court records & prior convictions
- 911 call recordings
- Weather data
- Media coverage

**Integration points** (bottom bar):
- VICAP | CODIS | AFIS | NIBIN | NamUs

---

## Slide 12: Ingestion — From Paper to Graph

**Title**: Digitizing Decades of Case Files

**Visual**: Pipeline diagram left to right:

```
Paper Files → OCR/Scanning → NLP Extraction → Entity Resolution → Case Graph
   │              │                │                  │                │
   │         Handwritten      Names, dates,      Dedup people,    Connected
   │         notes, typed     locations,         resolve aliases,  knowledge
   │         reports          relationships      link evidence     graph
```

**Bottom note**: "Initial MVP uses manual entry by investigators. Automated ingestion is Phase 2."

---

## Slide 13: Incoming Case Classification

**Title**: Not Just Cold Cases — Live Case Triage

**Visual**: Funnel diagram:

```
New Homicide Case
       ↓
  Solvability Scoring (initial evidence assessment)
       ↓
  MO Pattern Matching (HNSW similarity against solved cases)
       ↓
  Evidence Degradation Risk (time-sensitive lead flagging)
       ↓
  Resource Allocation Recommendation
       ↓
  Case Priority Score: 0-100
```

**Right side**: Example output card:
- Case #2026-1847
- Solvability: 73/100
- Similar to: 4 solved cases (MO match)
- Time-sensitive: Cell records expire in 90 days
- Recommendation: Assign to active investigation, subpoena cell records immediately

---

## Slide 14: Legal Foundation — Daubert Standard

**Title**: Built for Admissibility from Day One

**Visual**: Checklist with green checkmarks:

**Daubert v. Merrell Dow (1993) — Four Factors**:

- [x] **Testable methodology** — Causal graph construction, spectral analysis, and coherence scoring are mathematically defined and reproducible
- [x] **Peer review & publication** — Academic publication pathway via Northwestern partnership and computational criminology journals
- [x] **Known error rate** — Coherence scores have measurable precision; gap identification rates can be validated against solved cases
- [x] **General acceptance** — Bayesian networks already accepted in court for DNA evidence; causal graph methods are established in computer science

**Additional strength**: Florida adopted Daubert (Fla. Stat. section 90.702, effective 2013)

---

## Slide 15: Transparency & Trust

**Title**: Open Source, Auditable, Tamper-Evident

**Three columns**:

**Column 1 — ExoChain Audit Trail**
- Every analytical step logged
- Tamper-evident hash chain (SHAKE-256)
- Dual signatures (Ed25519 + ML-DSA-65)
- Complete reproducibility
- Icon: chain/lock

**Column 2 — Open Source Code**
- Full source code available for defense examination
- No "black box" algorithm challenges
- Unlike proprietary tools (Palantir, TrueAllele, Cellebrite)
- Icon: open book/code

**Column 3 — Brady Compliance**
- System inherently surfaces exculpatory evidence
- Gap analysis identifies untested alibi evidence
- Alternative suspect patterns automatically flagged
- Cannot be configured to suppress findings
- Icon: scales of justice

---

## Slide 16: Ethical Safeguards

**Title**: Designed to Prevent Harm

**Visual**: Shield icon, with four surrounding points:

1. **Bias detection** — System flags when evidence graph disproportionately targets based on demographic attributes
2. **Exculpatory priority** — Gaps that could exonerate are surfaced with equal or higher priority than incriminating gaps
3. **Human-in-the-loop** — System recommends, never decides. All investigative actions require human authorization
4. **Governance model** — Three-branch authorization (Legislative = SOPs, Executive = investigator action, Judicial = review board oversight)

**Bottom quote**: "The goal is not to build a case against someone. The goal is to find the truth — wherever it leads."

---

## Slide 17: MVP Prototype — One Case, Full Demonstration

**Title**: Proof of Concept Scope

**Visual**: Simple timeline/milestone diagram:

**Phase 1 — MVP (4-6 weeks)**
- One cold case selected by OPD
- Manual data entry by investigator
- Case Graph visualization with gap analysis
- Coherence scoring
- Competing hypothesis testing

**Phase 2 — Validation (2-3 months)**
- 5-10 cases (mix of cold + recently solved for validation)
- Compare system recommendations against actual investigative steps
- Measure: did the system identify the gaps that led to resolution?
- Academic publication of methodology and results

**Phase 3 — Integration (6+ months)**
- RMS/CAD data ingestion pipeline
- VICAP/CODIS integration
- Multi-case pattern analysis
- Department-wide deployment

---

## Slide 18: Partnership Model

**Title**: Who We Need at the Table

**Visual**: Four connected circles/logos:

**WeaveLogic** — Technology (ECC engine, causal graphs, gap analysis, visualization)
**Orlando PD** — Domain expertise, case data, investigator feedback, validation
**Northwestern University** — Academic rigor, research methodology, publication, wrongful conviction expertise
**UCF** — Local forensic science, student researchers, NCFS partnership

**Bottom**: "This is a research partnership — not a vendor relationship. We build it together."

---

## Slide 19: Funding & Sustainability

**Title**: Grant Opportunities & Long-Term Model

**Left column — Grant funding**:
- **NIJ** (National Institute of Justice) — Technology for criminal justice
- **BJA** (Bureau of Justice Assistance) — Smart Policing Initiative
- **NSF** — Computational social science
- **Arnold Ventures** — Criminal justice reform technology

**Right column — Long-term sustainability**:
- Pilot: grant-funded, zero cost to OPD
- Expansion: per-seat SaaS licensing ($X/investigator/month)
- Revenue: multi-department licensing as methodology is validated
- Academic: ongoing research grants + publications

---

## Slide 20: The Ask

**Title**: Next Steps

**Visual**: Three numbered boxes, left to right:

**1. Select a Case**
- OPD identifies one cold case suitable for pilot
- Case with substantial but disorganized evidence preferred
- Investigator assigned as domain expert partner

**2. Build the Prototype**
- WeaveLogic builds Case Graph for selected case
- 4-6 week development sprint
- Weekly check-ins with investigator for validation

**3. Evaluate Results**
- Did the system surface gaps the team hadn't considered?
- Does the coherence scoring match investigator intuition?
- Publish findings, plan Phase 2 expansion

**Bottom callout** (emerald accent, large text):
"Every day a case stays cold, evidence degrades and witnesses disappear. The questions that solve these cases already exist — we just need to ask them."

---

## Appendix Slides (Optional, for Q&A)

### Appendix A: Technical Architecture
- WeftOS kernel diagram
- ECC subsystem: CausalGraph, HNSW, DEMOCRITUS, CrossRef, ExoChain
- Data flow from ingestion to visualization

### Appendix B: Edge Type Taxonomy
- Full list of 15+ edge types with definitions
- Color coding scheme for visualization

### Appendix C: Comparison with Existing Tools
- Feature matrix: ECC vs i2 vs Palantir vs VICAP vs genetic genealogy
- Strengths/weaknesses of each

### Appendix D: Florida Legal Context
- Fla. Stat. section 90.702 (Daubert adoption)
- Recent Florida cases with technology-assisted investigation
- CJIS compliance requirements

### Appendix E: Orlando PD Context
- OPD size, cold case unit structure
- Orange County recent cold case successes
- Florida AG Cold Case Investigations Unit
