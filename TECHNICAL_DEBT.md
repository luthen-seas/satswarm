# Technical Debt and Known Issues

## Clippy Version Compatibility Issues

**Status**: Known issue - needs addressing in future iteration

**Problem**: 
- Project uses `nostr v0.35.0` directly
- `nostr-sdk v0.33.0` depends on `nostr v0.33.0` internally
- This creates incompatible types throughout the codebase
- Clippy reports 50+ compilation errors due to version conflicts

**Impact**:
- Code compiles and runs correctly
- Clippy analysis fails due to API mismatches
- Some deprecated APIs are used (Tag::generic, EventBuilder.sign_with_keys)

**Planned Resolution**:
1. Choose a single nostr version strategy:
   - Option A: Upgrade to nostr v0.35.0 + compatible nostr-sdk
   - Option B: Downgrade to nostr v0.33.0 throughout
2. Update API calls to match chosen version
3. Implement missing struct methods and types

**Workaround**: 
Development continues with clippy warnings disabled.

**Timeline**: Address after core functionality is stable.
