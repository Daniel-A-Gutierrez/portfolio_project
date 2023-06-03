OK what kind of site should i make

maybe a document database? 
on the homepage you basically just have a search bar with some options

ok i found a crate called pdf_extract that can give me plain strings from a pdf
i found another called axum::streambody that lets me send a stream as an http response body

can i visualize books as a graph between words? 
each character in a book has 2 properties - position, and value. 
it can be expressed as a *path*, or a sequence of edges, in a graph with nodes for every character.
each node stores a reference to the list of edges going from it 
each edge simply points to the next character, and previous edge ( need to be able to tell what path we are on)
at the end of the book we have a final node, which stores critical information about the book - title, isbn, etc. 

hows our performance looking like? 
well, to do a raw text search for a string of length A 
in a database with B books of length C

we need to search every character (B*C) for the first character of A 
we need to search each of those for the ones with the next character in A (should be 1/26)
and the next ... 

approx B*C , nearly linear for the length of the entire database. 
exactly a(1-r^n)/(1-r) if we assume each character string is equally likely (they arent) its B*C*log26(n). pretty good

for the graph storage we start out knowing all the places that start with the first character already. 
im assuming its 1node = 1 character but it could also be a string of characters if you use more nodes. nodes are N 
actually its logN(B*C)*C/N + C i think. Dam thats fast for a full text search. 
If instead i use, say, the top 1k most common words it becomes log1000(B*C)*(C/1000) + C 
realistically the size and speed of the database will decrease too as nodes increases even if each node is just 2 sequential characters. 

So how big is this database? 
edgesize * edges + nodesize * nodes

increasing nodes by linking common sequential edges speeds everything up here.

nodes should be kept in a sorted array and be constant length.
edge lookups within a nodes edge list are the most expensive operation - though sorting them isnt too hard. 

books : B
booklen : Lb
characters : Ch - number of characters represented as nodes. 255 is easy. 
chunklen (in nodes): Lc
nodelen : Ln (how many characters 1 node represents)
nodes : N  <=  Ch^Ln.



edge : 
next node * : log2(N) < log2(Ch^Ln) = log2(Ch)*Ln = 8*Ln so Ln Bytes at most. //pointer to next node. since Ch is at least 255. 
edge * : log2() // or actually i guess just an index is better, that makes the operation o(1) and we dont need to bother sorting. cool.  
                // yeah i guess just the index of the next edge in the next node that corresponds to the next character in this book chunk. 
                // technically this is unbounded, unless we limit the number of books per library. 
                // make it a u32 and say the library is full when some edgelist grows to be over 2B elements long?
                // or a u16 and likewise, 65k? More nodes would help. Max library content (in edges)= (2^edge* size)*Ch^Ln * Ch characters
                // so even with just a u16 for Node's value (Ch*Ln = log2(255)*2) and Edge* being a u16 , we get 65k * 65k * 255 = 2^40, or 1.1 trillion chars.
                //an average book has half a million characters so thats great, the most common word takes up about 2%, so more realistically,
                //its about 50 x 2^(edge* len) . could get like 10x more by ignoring 10 common words. 
                //lets just say then an edge takes 4 bytes, that means my graph is MINIMUM 4x the size of the library. 
node : 
edge[] 
value = log2(Ch) = u8
index : an index for the alphebatized indeces of the edges

nodes : 
node[]

endings:
ending[]

ending:
book,
chunknum

each character is a node, edges are a reference to a node and another edge so lets say a u64 + u16.
edges can also be a reference to a end struct, which is just a pointer so a u64. 
so while a character is usually only 1 byte, each edge is minimum 9 , probably padded to 16. I bet i could cut that in half
by using a u32 instead, but the max book length then is 4 billion characters. If I split books into chunks thats fine. just more end nodes. 
infact, i should probably just have more end nodes, and split books into chunks that fit into a u16. 


actually an index is really valuable here, even just knowing the locations of the first word helps alot.  

honestly actually fuckit, its probably more efficient to make each node just a word. theres only like 10k in english so a u16 is fine, its an index into an 
array of all the words. now each edge * is also a u16 and itll point to one of 65k occurrances of the word. 65k * 10k * 4 Bytes is 2.6GB. 
Fuckit ill make it a u32, 4 billion occurances now so our max size is now 65k*65k*10k*6 is 253 TB. Realistically itll let us insert 4.294B * 50 = 200 billion words before it craps out, which would be 200GB * 2 = 400GB.

yeah i mean its fine in ram for now but like, it should be in a database on a disk.
hey actually why not? Make an edge list an entry in postgres that is. 

actually the pdfs themselves kindof have to exist as files, so postgres i guess mainly holds info to search them. that works for me. 

should use a path builder struct that stores the current (previously inserted) character, the previous, and can be given the next.
also stores index of last edge in previous node. 

2 special nodes : 0x00 and 0xff. these are chunk beginnings and ends. 
within those nodes edge lists a reference to 0 means the beginning of the book. So both need a pad for the first element.
all paths have the same index in the edge list of 0x00 and 0xff since all chunks have 1 beginning and 1 end, and the next chunk 
is always at index + 1. a separate data exists that maps the indeces to a bookinfo struct to which the chunk belongs. 

i think for the sake of speed its better to have a bunch of small libraries rather than one big library we have to operate on from the disk. 
also if instead of storing an array of edges we stored an array of chunks of edges we can probably speed up searching quite a bit and increase
capacity. 

struct edgeref { fromNode , index } 
struct edge { toNode , prevIndex }
struct bookinfo { book name, isbn, author }
capstones < edgeIndex, capstone> : map
struct pathBuilder { current : edgeref , prev : edgeref, ending:capstone }
struct book { bookinfo, contents }

pathBuilder::new(book)
    
    startChunkIndex = lib.nodes[start].length;

    //split book into 1KB long chunks (improves look up time, minimally affects capacity/efficiency)
    for chunk,i in chunks.enumerate():
        chunkIndex = lib.nodes[start].length - startChunkIndex;
        firstLetterIndex = chunkIndex*chunkSize;
        firstLetter = book.contents[firstLetterIndex]

        current = edgeref{firstLetter, lib.nodes[firstLetter].length}
        prev = edgeref{start, lib.nodes[start].length}
        lib.nodes[start].pushEdge( edge{firstLetter,0} )
        for next,i2 in chunk.enumerate() :
            currentEdgeIndex = lib.nodes[current.fromNode].length //kinda wasteful
            lib.nodes[current.fromNode].pushEdge( edge {next, current.index} )
            prev = current
            current = edgeref{ next , lib.nodes[next].length } //or something like that.
