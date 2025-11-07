# README

## Project Description

A cross-platform disk usage visualizer built in Rust, designed to scan directories, visualize file sizes, and manage duplicates efficiently.

The project is drive by multiple dependencies.

| **Dependency** | **Purpose** |
| --- | --- |
| **Iced GUI** | Handles all graphical user interface elements |
| **WalkDir** | Scans through all files and subfolders in a directory |
| **RFD** | Opens native file and folder selection dialogs |
| **trash** | Moves files to the recycle bin safely |
| **blake3** | Generates unique file fingerprints using hashing |
| **tokio** | Runs multiple tasks concurrently without slowing down |

## Supported Operating Systems

| Operating Systems |
| - |
| ☑ Linux |
| ☑ Window |
| ☑ MacOS |