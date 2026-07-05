# P5R-d Notes

TargetActionKrylov algebra now excludes debug explicit handles from production function signatures and exposes `require_production_target_action_krylov_handle` for explicit boundary rejection.

Production handles must be built from authorized relations, per-basis normal-form certificates, and independent normal-form membership certificates for each action column. Certificate hashes are recomputed during validation and tamper tests cover authorization, action certificate hashes, and basis certificate hashes.
