# Remote fold: version 2

## What changed from version 1

- The Mac is the control desk and the Oracle box is the workshop; install day on the Mac comes first.
- Box updates use the same in-place binary swap and live process handoff as the Mac, so running lanes keep their terminals.
- Each box pane receives the guarded tool first on its own path; no login-shell assumption remains.
- The Mac courier brings back completion records and report bytes, then tells the coordinator when a box lane is blocked or gone.
- The box gets its own narrow GitHub login and commit identity, while the Mac selects the destination by its address rather than its nickname.
- Disk is counted per lane at five to ten gigabytes, with one build folder per lane and a twelve-gigabyte start gate.
- The box server starts at boot; lanes interrupted by a reboot are reported gone and restarted from their job card.
- Project mappings and rule text travel in repositories; no home-folder file or login is copied between machines.

## Choices for Rolf

1. **Who can see the two repositories before remote work starts?**
   - **Private — my reading:** only invited people can read job briefs and lane branches.
   - **Public:** anyone can read every pushed brief and lane branch, as they can today.

2. **How far may the box's GitHub key reach?**
   - **Only the two tool repositories — my reading:** later projects are added one at a time when you choose them.
   - **Every repository you own:** the box can publish anywhere without another setup step.

3. **How should you handle disk space for Rust lanes?**
   - **Keep the current disk with the start gate — my reading:** the doctor shows how many isolated lanes fit and refuses the next one before space becomes unsafe.
   - **Grow the boot disk now:** one Oracle volume-size change buys room before the first lane.
   - **Keep Rust work on the Mac:** the box runs only jobs that do not create large build folders.

4. **Which other project repositories should be on the box on day one?**
   - **None — my reading:** add a project yourself only when its first remote lane is needed.
   - **All active projects:** clone every current project during setup so first starts never pause for preparation.

5. **Should the box keep its locked public address for outbound traffic?**
   - **Keep it for now — my reading:** inbound stays closed and the address supplies package, model, web, and GitHub traffic.
   - **Add a paid network hallway later:** remove the public address after the new outbound route is working.

6. **Which jobs may use the DeepSeek and Muse helpers?**
   - **Lane work only — my reading:** add each helper only to the ordinary worker list.
   - **Lane and review work:** reviewers may use the same helpers when you name them.
   - **Decide per project:** every project keeps its own allowed-job list.

7. **How should a Linux box lane handle the Windows check?**
   - **Install the cross-build kit once — my reading:** a box lane runs the same complete check as a Mac lane.
   - **Keep those code lanes on the Mac:** no required check is silently skipped on the box.

8. **Who cleans a reviewed box worktree and its build folder?**
   - **The Mac orders cleanup after review — my reading:** the box deletes locally only after the report and release gates pass.
   - **The lane cleans its own bench:** each worker removes its folder before finishing.
