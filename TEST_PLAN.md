# Test Result

| Test Case| Description| Expected Outcome| Result   
| --- | --- | --- | --- |
| **1. Folder Selection** | User clicks **Browse** and picks a directory | The app displays selected folder path  | ☑ Passed |
| **2. Folder Scanning** | Scans directory and lists files sorted by size| Files appear correctly with size bars  | ☑ Passed |
| **3. Folder Aggregation**| Aggregates subfolder sizes correctly| Folder list shows accurate sizes | ☑ Passed |
| **4. Delete File**| User clicks “Delete”| File moves to Trash and list refreshes | ☑ Passed |
| **5. Make Duplicate**| Creates a copy of a file with `_copy` suffix| Duplicate file appears in list | ☑ Passed |
| **6. Move File** | Select destination and move file | File moves to new folder successfully | ☑ Passed |
| **7. Duplicate Detection** | Detect identical files by content (Blake3 hash) | Groups of duplicates displayed | ☑ Passed |
| **8. Delete Duplicates** | Deletes all files in duplicate group | Duplicates removed, view updates | ☑ Passed |
| **9. Navigation Between Screens** | Navigate Home → Visualization → Duplicates | No UI freeze or data loss | ☑ Passed |
| **10. Exit Application** | Click “Exit” | Application terminates cleanly | ☑ Passed|