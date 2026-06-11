// UI preset experiments (Stage 5): the validation suite's configs-of-record,
// embedded so they run identically native and on wasm32 (no filesystem).
//
// Files in ../presets/ are exported by validation/export_ui_presets.py from
// validation/scenarios/ — never hand-edit them here; re-run the exporter.
// Each preset is the EXACT scenario the harness ran for the corresponding
// validation gate (or demo video), so what the UI shows is what was tested.

pub struct Preset {
    pub name: &'static str,
    pub blurb: &'static str,
    pub json: &'static str,
}

pub const PRESETS: &[Preset] = &[
    Preset {
        name: "T0 — free fall",
        blurb: "Single disk, gravity only. Stage-0 harness check: trajectory matches y = y0 - g t^2/2 to 0.01%.",
        json: include_str!("../presets/t0_verify.json"),
    },
    Preset {
        name: "T1 — two-disk normal impact",
        blurb: "Head-on impact, linear spring contact. Overlap and velocities match the closed form to <0.1%.",
        json: include_str!("../presets/t1_normal_impact.json"),
    },
    Preset {
        name: "T2 — wall bounce",
        blurb: "As-built impulsive wall law: v -> -v/2, spin unchanged.",
        json: include_str!("../presets/t2_wall_bounce.json"),
    },
    Preset {
        name: "T3 — restitution (dashpot \u{3b2}=0.2)",
        blurb: "Stage-3b viscous contact dashpot. Rebound speed matches the no-tension spring-dashpot ODE (RMS 3e-4).",
        json: include_str!("../presets/t3d_b0.2.json"),
    },
    Preset {
        name: "T4 — oblique impact at the stick/slip kink (\u{3c8}=42\u{b0})",
        blurb: "Tangential spring + Coulomb cap, right at the predicted stick/slip transition angle.",
        json: include_str!("../presets/t4_psi42.json"),
    },
    Preset {
        name: "T5 — slide \u{2192} roll transition",
        blurb: "Spinning disk on a frictional floor: slides, then sticks to pure rolling at t* = v0/3\u{3bc}g.",
        json: include_str!("../presets/t5_rolling.json"),
    },
    Preset {
        name: "T6 — bonded cantilever (N=20)",
        blurb: "Parallel-bonded chain, full gravity: swings down and rings before parking. (The VALIDATED gate config uses 0.002g and deflects 0.05 units — sub-pixel by design; tip matches the discrete beam solution to <0.7%.)",
        json: include_str!("../presets/t6_demo_g1.json"),
    },
    Preset {
        name: "T7 — gravity settling, N\u{2248}1200",
        blurb: "The Stage-2/3 instability test, at the production granular recipe (\u{3b2}=0.2, \u{3b1}=0.05). Settles to KE/peak ~1e-6.",
        json: include_str!("../presets/t7d_N1024_b0.2_a0.05.json"),
    },
    Preset {
        name: "T7b — bonded block drop + fracture, N\u{2248}1200",
        blurb: "Bonded block falls ballistically and fractures on impact (nodamp recipe: \u{3b2}=0.2, \u{3b1}=0). Energy stays monotone through the cascade.",
        json: include_str!("../presets/t7bd_N1024_nodamp.json"),
    },
    Preset {
        name: "T8 — silo drain \u{2192} angle of repose (\u{3bc}=0.5)",
        blurb: "Stage-4 bulk test: drain a silo, measure the heap angle. N\u{2248}4700.",
        json: include_str!("../presets/t8_video_mu0.5.json"),
    },
    Preset {
        name: "T9 — packing fraction rain (\u{3bc}=0.3, N\u{2248}10k)",
        blurb: "Stage-4 bulk test: random rain into a box; packing fraction lands in the 2D random-close-packing band.",
        json: include_str!("../presets/t9d_mu0.3_s1.json"),
    },
    Preset {
        name: "T10 — oedometer crush (N\u{2248}3300)",
        blurb: "Stage-4 bulk test: confined compression by a moving plate; modulus scales with contact stiffness.",
        json: include_str!("../presets/t10_video_kn2e8.json"),
    },
    Preset {
        name: "Demo — unconfined bonded crush",
        blurb: "Bonded polydisperse specimen, no side walls, plate from above: elastic, then catastrophic brittle collapse with lateral calving.",
        json: include_str!("../presets/vidcr_full_s3.5e+07.json"),
    },
    Preset {
        name: "Demo — polydisperse bonded drop",
        blurb: "Bonded polydisperse block dropped 30 m: fractures into large intact fragments on a rubble bed.",
        json: include_str!("../presets/viddp_drop_s1.2e+07.json"),
    },
];
