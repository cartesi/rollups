# Advance Runner

This service consumes rollups input events from the broker and use them to advance the server-manager state.
When the epoch finishes, the advance-runner gets the claim from the server-manager and produces the rollups claim event.
