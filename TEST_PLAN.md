# Test Result

| Test Case| Description| Expected Outcome| Result   
| --- | --- | --- | --- |
| **1. Folder Selection** | User clicks **Browse** and picks a directory | The app displays selected folder path  | - [x] Passed |
| **2. Folder Scanning** | Scans directory and lists files sorted by size| Files appear correctly with size bars  | - [x] Passed |
| **3. Folder Aggregation**| Aggregates subfolder sizes correctly| Folder list shows accurate sizes | - [x] Passed |
| **4. Delete File**| User clicks “Delete”| File moves to Trash and list refreshes | - [x] Passed |
| **5. Make Duplicate**| Creates a copy of a file with `_copy` suffix| Duplicate file appears in list | - [x] Passed |
| **6. Move File** | Select destination and move file | File moves to new folder successfully | - [x] Passed |
| **7. Duplicate Detection** | Detect identical files by content (Blake3 hash) | Groups of duplicates displayed | - [x] Passed |
| **8. Delete Duplicates** | Deletes all files in duplicate group | Duplicates removed, view updates | - [x] Passed |
| **9. Navigation Between Screens** | Navigate Home → Visualization → Duplicates | No UI freeze or data loss | - [x] Passed |
| **10. Exit Application** | Click “Exit” | Application terminates cleanly | - [x] Passed|