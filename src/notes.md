# Important Info

Inodes saved as JSON String in Block data

Blocks saved as JSON to file Line

Inode 0 is the root directory file.

Only one directory.

What is a Free Block? = If Data = None

What is a Free Inode? = If Inode_Type is Free

Example of Inode table
{nextNode: None, data: "[{number: 1, i_type: Directory, start_block: 3}]" }

Each Inode table should hold 5 inodes

# ASSUMPTIONS

All inodes exists in the file

# Use Cases

_Saving a file_

1. Read file content into a String
1. Chunk up into 1024 bytes Array of Strings
1. Get First Free block, create new inode pointing to that blocks number.
1. Continuously get the next free block pointing to the previous one until out of byte array of data.
1. Save Inode and block to disk.
1. Update Root Inode to include the fileName with the inode number within it's content

_Reading a files data_

1. Pass in fileName
1. Search directory inode for fileName inode number
1. Take inode number and get associated inode
1. Construct inode data from startBlock number
1. Return construct data by reading link list of startblocks to next blocks

_Printing directory_

1. Read Inode 0
1. Construct data by reading linked list
1. Return String of the content

_Delete a file_

1. Find Inode of given file by reading Inode 0
1. Set Inode to free
1. Set associated blocks to free
1. Remove this fileName and inode number from directory
1. Save Inode list back within Inode table Block
