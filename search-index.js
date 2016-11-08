var searchIndex = {};
searchIndex["algorithmia"] = {"doc":"Algorithmia client library","items":[[0,"mime","algorithmia","Re-exporting the mime crate, for convenience.",null,null],[3,"Mime","algorithmia::mime","Mime, or Media Type. Encapsulates common registers types.",null,null],[12,"0","","",0,null],[12,"1","","",0,null],[12,"2","","",0,null],[4,"SubLevel","","",null,null],[13,"Star","","",1,null],[13,"Plain","","",1,null],[13,"Html","","",1,null],[13,"Xml","","",1,null],[13,"Javascript","","",1,null],[13,"Css","","",1,null],[13,"EventStream","","",1,null],[13,"Json","","",1,null],[13,"WwwFormUrlEncoded","","",1,null],[13,"Msgpack","","",1,null],[13,"OctetStream","","",1,null],[13,"FormData","","",1,null],[13,"Png","","",1,null],[13,"Gif","","",1,null],[13,"Bmp","","",1,null],[13,"Jpeg","","",1,null],[13,"Ext","","",1,null],[4,"TopLevel","","",null,null],[13,"Star","","",2,null],[13,"Text","","",2,null],[13,"Image","","",2,null],[13,"Audio","","",2,null],[13,"Video","","",2,null],[13,"Application","","",2,null],[13,"Multipart","","",2,null],[13,"Message","","",2,null],[13,"Model","","",2,null],[13,"Ext","","",2,null],[4,"Value","","",null,null],[13,"Utf8","","",3,null],[13,"Ext","","",3,null],[4,"Attr","","",null,null],[13,"Charset","","",4,null],[13,"Boundary","","",4,null],[13,"Q","","",4,null],[13,"Ext","","",4,null],[6,"Param","","",null,null],[3,"Url","algorithmia","A parsed URL record.",null,null],[3,"Algorithmia","","The top-level struct for instantiating Algorithmia client endpoints",null,null],[0,"algo","","Interact with Algorithmia algorithms",null,null],[3,"Algorithm","algorithmia::algo","Algorithmia algorithm - intialized from the `Algorithmia` builder",null,null],[12,"path","","",5,null],[3,"AlgoOptions","","Options used to alter the algorithm call, e.g. configuring the timeout",null,null],[3,"AlgoRef","","",null,null],[12,"path","","",6,null],[3,"AlgoMetadata","","Metadata returned from the API",null,null],[12,"duration","","",7,null],[12,"stdout","","",7,null],[12,"alerts","","",7,null],[12,"content_type","","",7,null],[3,"AlgoResponse","","Successful API response that wraps the AlgoOutput and its Metadata",null,null],[12,"metadata","","",8,null],[12,"result","","",8,null],[4,"AlgoInput","","Types that can be used as input to an algorithm",null,null],[13,"Text","","Data that will be sent with `Content-Type: text/plain`",9,null],[13,"Binary","","Data that will be sent with `Content-Type: application/octet-stream`",9,null],[13,"Json","","Data that will be sent with `Content-Type: application/json`",9,null],[4,"AlgoOutput","","Types that can store the output of an algorithm",null,null],[13,"Text","","Representation of result when `metadata.content_type` is &#39;text&#39;",10,null],[13,"Json","","Representation of result when `metadata.content_type` is &#39;json&#39;",10,null],[13,"Binary","","Representation of result when `metadata.content_type` is &#39;binary&#39;",10,null],[4,"Version","","Version of an algorithm",null,null],[13,"Latest","","Latest published version",11,null],[13,"Minor","","Latest published version with the same minor version, e.g., 1.2 implies 1.2.*",11,null],[13,"Revision","","A specific published revision, e.g., 0.1.0",11,null],[13,"Hash","","A specific git hash - only works for the algorithm&#39;s author",11,null],[11,"decode","","",7,{"inputs":[{"name":"__d"}],"output":{"name":"result"}}],[11,"fmt","","",7,null],[11,"new","","",5,{"inputs":[{"name":"httpclient"},{"name":"algoref"}],"output":{"name":"algorithm"}}],[11,"to_url","","Get the API Endpoint URL for this Algorithm",5,null],[11,"to_algo_uri","","Get the Algorithmia algo URI for this Algorithm",5,null],[11,"pipe","","Execute an algorithm with",5,null],[11,"pipe_json","","Execute an algorithm with explicitly set content-type",5,null],[11,"pipe_as","","",5,null],[11,"set_options","","Builder method to explicitly configure options",5,null],[11,"timeout","","Builder method to configure the timeout in seconds",5,null],[11,"enable_stdout","","",5,null],[11,"as_string","","If the `AlgoInput` is text (or a valid JSON string), returns the associated text",9,null],[11,"as_json","","If the `AlgoInput` is Json (or text that can be JSON encoded), returns the associated JSON string",9,null],[11,"as_bytes","","If the `AlgoInput` is binary, returns the associated byte slice",9,null],[11,"decode","","If the `AlgoInput` is valid JSON, decode it to a particular type",9,null],[11,"as_string","","If the result is text (or a valid JSON string), returns the associated string",8,null],[11,"as_json","","If the result is Json (or text that can be JSON encoded), returns the associated JSON string",8,null],[11,"as_bytes","","If the result is Binary, returns the associated byte slice",8,null],[11,"decode","","If the result is valid JSON, decode it to a particular type",8,null],[11,"new","","",12,{"inputs":[],"output":{"name":"algooptions"}}],[11,"timeout","","",12,null],[11,"enable_stdout","","",12,null],[11,"deref","","",12,null],[11,"deref_mut","","",12,null],[11,"from_str","","",8,{"inputs":[{"name":"str"}],"output":{"name":"result"}}],[11,"fmt","","",8,null],[11,"read","","",8,null],[11,"from","","",6,{"inputs":[{"name":"str"}],"output":{"name":"self"}}],[11,"from","","",6,null],[11,"from","","",9,null],[11,"from","","",9,{"inputs":[{"name":"str"}],"output":{"name":"self"}}],[11,"from","","",9,null],[11,"from","","",9,{"inputs":[{"name":"string"}],"output":{"name":"self"}}],[11,"from","","",9,{"inputs":[{"name":"vec"}],"output":{"name":"self"}}],[11,"from","","",9,{"inputs":[{"name":"json"}],"output":{"name":"self"}}],[11,"from","","",9,{"inputs":[{"name":"e"}],"output":{"name":"self"}}],[11,"from","","",10,null],[11,"from","","",10,{"inputs":[{"name":"str"}],"output":{"name":"self"}}],[11,"from","","",10,{"inputs":[{"name":"string"}],"output":{"name":"self"}}],[11,"from","","",10,null],[11,"from","","",10,{"inputs":[{"name":"vec"}],"output":{"name":"self"}}],[11,"from","","",10,{"inputs":[{"name":"json"}],"output":{"name":"self"}}],[11,"from","","",10,{"inputs":[{"name":"e"}],"output":{"name":"self"}}],[11,"from","","",9,{"inputs":[{"name":"algooutput"}],"output":{"name":"self"}}],[11,"from","","",11,{"inputs":[{"name":"str"}],"output":{"name":"self"}}],[11,"from","","",11,null],[11,"from","","",11,null],[11,"fmt","","",11,null],[8,"DecodedEntryPoint","","Alternate implementation for `EntryPoint`\n  that automatically decodes JSON input to the associate type.",null,null],[16,"Input","","",13,null],[10,"apply_decoded","","This method is an apply variant that will receive the decoded form of JSON input.\nIf decoding failed, a `DecoderError` will be returned before this method is invoked.",13,null],[8,"EntryPoint","","Implementing an algorithm involves overriding at least one of these methods",null,null],[11,"apply_str","","",14,null],[11,"apply_json","","",14,null],[11,"apply_bytes","","",14,null],[11,"apply","","The default implementation of this method calls\n`apply_str`, `apply_json`, or `apply_bytes` based on the input type.",14,null],[0,"data","algorithmia","Manage data for algorithms",null,null],[3,"XDataType","algorithmia::data","",null,null],[12,"0","","",15,null],[3,"XErrorMessage","","",null,null],[12,"0","","",16,null],[3,"DeletedResult","","",null,null],[12,"deleted","","",17,null],[4,"DataType","","",null,null],[13,"File","","",18,null],[13,"Dir","","",18,null],[4,"DataObject","","",null,null],[13,"File","","",19,null],[13,"Dir","","",19,null],[5,"parse_data_uri","","",null,{"inputs":[{"name":"str"}],"output":{"name":"str"}}],[0,"dir","","Directory module for managing Algorithmia Data Directories",null,null],[3,"DataDir","algorithmia::data::dir","Algorithmia Data Directory",null,null],[3,"DirectoryUpdated","","",null,null],[12,"acl","","",20,null],[3,"DirectoryDeleted","","Response when deleting a new Directory",null,null],[12,"result","","",21,null],[3,"DataAcl","","ACL that indicates permissions for a DataDirectory\nSee also: [ReadAcl](enum.ReadAcl.html) enum to construct a DataACL",null,null],[12,"read","","",22,null],[3,"DirectoryListing","","",null,null],[12,"acl","","",23,null],[3,"DataFileEntry","","",null,null],[12,"size","","",24,null],[12,"last_modified","","",24,null],[4,"ReadAcl","","",null,null],[13,"Private","","Readable only by owner",25,null],[13,"MyAlgorithms","","Readable by owner&#39;s algorithms (regardless of caller)",25,null],[13,"Public","","Readable by any user",25,null],[4,"DirEntry","","",null,null],[13,"File","","",26,null],[13,"Dir","","",26,null],[11,"decode","","",20,{"inputs":[{"name":"__d"}],"output":{"name":"result"}}],[11,"fmt","","",20,null],[11,"decode","","",21,{"inputs":[{"name":"__d"}],"output":{"name":"result"}}],[11,"fmt","","",21,null],[11,"decode","","",22,{"inputs":[{"name":"__d"}],"output":{"name":"result"}}],[11,"encode","","",22,null],[11,"fmt","","",22,null],[11,"default","","",22,{"inputs":[],"output":{"name":"self"}}],[11,"from","","",22,{"inputs":[{"name":"readacl"}],"output":{"name":"self"}}],[11,"deref","","",24,null],[11,"next","","",23,null],[11,"new","","",27,{"inputs":[{"name":"httpclient"},{"name":"str"}],"output":{"name":"self"}}],[11,"path","","",27,null],[11,"client","","",27,null],[11,"list","","Display Directory details if it exists",27,null],[11,"create","","Create a Directory",27,null],[11,"delete","","Delete a Directory",27,null],[11,"put_file","","Upload a file to an existing Directory",27,null],[11,"child","","",27,null],[0,"file","algorithmia::data","File module for managing Algorithmia Data Files",null,null],[3,"FileAdded","algorithmia::data::file","Response when creating a file via the Data API",null,null],[12,"result","","",28,null],[3,"FileDeleted","","Response when deleting a file from the Data API",null,null],[12,"result","","",29,null],[3,"DataResponse","","",null,null],[3,"DataFile","","Algorithmia data file",null,null],[11,"decode","","",28,{"inputs":[{"name":"__d"}],"output":{"name":"result"}}],[11,"fmt","","",28,null],[11,"decode","","",29,{"inputs":[{"name":"__d"}],"output":{"name":"result"}}],[11,"fmt","","",29,null],[11,"read","","",30,null],[11,"new","","",31,{"inputs":[{"name":"httpclient"},{"name":"str"}],"output":{"name":"self"}}],[11,"path","","",31,null],[11,"client","","",31,null],[11,"put","","Write to the Algorithmia Data API",31,null],[11,"get","","Get a file from the Algorithmia Data API",31,null],[11,"delete","","Delete a file from from the Algorithmia Data API",31,null],[0,"path","algorithmia::data","",null,null],[3,"DataPath","algorithmia::data::path","",null,null],[8,"HasDataPath","","",null,null],[10,"new","","",32,{"inputs":[{"name":"httpclient"},{"name":"str"}],"output":{"name":"self"}}],[10,"path","","",32,null],[10,"client","","",32,null],[11,"to_url","","Get the API Endpoint URL for a particular data URI",32,null],[11,"to_data_uri","","Get the Algorithmia data URI a given Data Object",32,null],[11,"parent","","Get the parent off a given Data Object",32,null],[11,"basename","","Get the basename from the Data Object&#39;s path (i.e. unix `basename`)",32,null],[11,"exists","","Determine if a file or directory exists for a particular data URI",32,null],[11,"new","","",33,{"inputs":[{"name":"httpclient"},{"name":"str"}],"output":{"name":"self"}}],[11,"path","","",33,null],[11,"client","","",33,null],[11,"get_type","","Determine if a particular data URI is for a file or directory",33,null],[11,"into_type","","",33,null],[11,"from","algorithmia::data::dir","",27,{"inputs":[{"name":"datapath"}],"output":{"name":"self"}}],[11,"from","algorithmia::data::file","",31,{"inputs":[{"name":"datapath"}],"output":{"name":"self"}}],[11,"clone","algorithmia::data","",15,null],[11,"fmt","","",15,null],[11,"eq","","",15,null],[11,"ne","","",15,null],[11,"deref","","",15,null],[11,"deref_mut","","",15,null],[11,"header_name","","",15,{"inputs":[],"output":{"name":"str"}}],[11,"parse_header","","",15,null],[11,"fmt_header","","",15,null],[11,"fmt","","",15,null],[11,"clone","","",16,null],[11,"fmt","","",16,null],[11,"eq","","",16,null],[11,"ne","","",16,null],[11,"deref","","",16,null],[11,"deref_mut","","",16,null],[11,"header_name","","",16,{"inputs":[],"output":{"name":"str"}}],[11,"parse_header","","",16,null],[11,"fmt_header","","",16,null],[11,"fmt","","",16,null],[11,"decode","","",17,{"inputs":[{"name":"__d"}],"output":{"name":"result"}}],[11,"fmt","","",17,null],[0,"error","algorithmia","Error types",null,null],[3,"ApiError","algorithmia::error","",null,null],[12,"message","","",34,null],[12,"stacktrace","","",34,null],[3,"ApiErrorResponse","","Struct for decoding Algorithmia API error responses",null,null],[12,"error","","",35,null],[4,"Error","","Errors that may be returned by this library",null,null],[13,"ApiError","","",36,null],[13,"ContentTypeError","","",36,null],[13,"DataTypeError","","",36,null],[13,"DataPathError","","",36,null],[13,"HttpError","","",36,null],[13,"DecoderError","","",36,null],[13,"EncoderError","","",36,null],[13,"FromBase64Error","","",36,null],[13,"IoError","","",36,null],[13,"Utf8Error","","",36,null],[13,"UnsupportedInput","","",36,null],[5,"decode","","",null,{"inputs":[{"name":"str"}],"output":{"name":"error"}}],[11,"fmt","","",36,null],[11,"decode","","",34,{"inputs":[{"name":"__d"}],"output":{"name":"result"}}],[11,"fmt","","",34,null],[11,"decode","","",35,{"inputs":[{"name":"__d"}],"output":{"name":"result"}}],[11,"fmt","","",35,null],[11,"description","","",34,null],[11,"description","","",36,null],[11,"cause","","",36,null],[11,"fmt","","",36,null],[11,"fmt","","",34,null],[11,"from","","",36,{"inputs":[{"name":"apierror"}],"output":{"name":"error"}}],[11,"from","","",36,{"inputs":[{"name":"error"}],"output":{"name":"error"}}],[11,"from","","",36,{"inputs":[{"name":"error"}],"output":{"name":"error"}}],[11,"from","","",36,{"inputs":[{"name":"decodererror"}],"output":{"name":"error"}}],[11,"from","","",36,{"inputs":[{"name":"encodererror"}],"output":{"name":"error"}}],[11,"from","","",36,{"inputs":[{"name":"frombase64error"}],"output":{"name":"error"}}],[11,"from","","",36,{"inputs":[{"name":"utf8error"}],"output":{"name":"error"}}],[11,"from","","",36,{"inputs":[{"name":"fromutf8error"}],"output":{"name":"error"}}],[0,"client","algorithmia","Internal client",null,null],[3,"Response","algorithmia::client","A response for a client request to a remote server.",null,null],[12,"status","","The status from the server.",37,null],[12,"headers","","The headers from the server.",37,null],[12,"version","","The HTTP version of this response from the server.",37,null],[12,"url","","The final URL of this response.",37,null],[4,"Body","","An enum of possible body types for a Request.",null,null],[13,"ChunkedBody","","A Reader does not necessarily know it&#39;s size, so it is chunked.",38,null],[13,"SizedBody","","For Readers that can know their size, like a `File`.",38,null],[13,"BufBody","","A String has a size, and uses Content-Length.",38,null],[3,"HttpClient","","Internal HttpClient to build requests: wraps `hyper` client",null,null],[12,"base_url","","",39,null],[4,"ApiAuth","","Represent the different ways to auth with the API",null,null],[13,"SimpleAuth","","",40,null],[13,"NoAuth","","",40,null],[11,"clone","","",40,null],[11,"new","","Instantiate an HttpClient - creates a new `hyper` client",39,{"inputs":[{"name":"apiauth"},{"name":"string"}],"output":{"name":"httpclient"}}],[11,"get","","Helper to make Algorithmia GET requests with the API key",39,null],[11,"head","","Helper to make Algorithmia GET requests with the API key",39,null],[11,"post","","Helper to make Algorithmia POST requests with the API key",39,null],[11,"put","","Helper to make Algorithmia PUT requests with the API key",39,null],[11,"delete","","Helper to make Algorithmia POST requests with the API key",39,null],[11,"clone","","",39,null],[11,"from","","",40,{"inputs":[{"name":"str"}],"output":{"name":"self"}}],[11,"from","","",40,null],[11,"client","algorithmia","Instantiate a new client",41,{"inputs":[{"name":"a"}],"output":{"name":"algorithmia"}}],[11,"alt_client","","Instantiate a new client against alternate API servers",41,{"inputs":[{"name":"url"},{"name":"a"}],"output":{"name":"algorithmia"}}],[11,"algo","","Instantiate an [`Algorithm`](algo/algorithm.struct.html) from this client",41,null],[11,"dir","","Instantiate a `DataDirectory` from this client",41,null],[11,"file","","Instantiate a `DataDirectory` from this client",41,null],[11,"data","","Instantiate a `DataPath` from this client",41,null],[11,"clone","","",41,null],[11,"default","","",41,{"inputs":[],"output":{"name":"algorithmia"}}],[11,"get_param","algorithmia::mime","",0,null],[11,"eq","","",0,null],[11,"eq","","",2,null],[11,"eq","","",2,null],[11,"eq","","",2,null],[11,"eq","","",2,null],[11,"eq","","",1,null],[11,"eq","","",1,null],[11,"eq","","",1,null],[11,"eq","","",1,null],[11,"eq","","",4,null],[11,"eq","","",4,null],[11,"eq","","",4,null],[11,"eq","","",4,null],[11,"eq","","",3,null],[11,"eq","","",3,null],[11,"eq","","",3,null],[11,"eq","","",3,null],[11,"from_str","","",0,{"inputs":[{"name":"str"}],"output":{"name":"result"}}],[11,"from_str","","",2,{"inputs":[{"name":"str"}],"output":{"name":"result"}}],[11,"from_str","","",1,{"inputs":[{"name":"str"}],"output":{"name":"result"}}],[11,"from_str","","",4,{"inputs":[{"name":"str"}],"output":{"name":"result"}}],[11,"from_str","","",3,{"inputs":[{"name":"str"}],"output":{"name":"result"}}],[11,"fmt","","",0,null],[11,"fmt","","",2,null],[11,"fmt","","",1,null],[11,"fmt","","",4,null],[11,"fmt","","",3,null],[11,"deref","","",2,null],[11,"deref","","",1,null],[11,"deref","","",4,null],[11,"deref","","",3,null],[11,"clone","","",0,null],[11,"clone","","",2,null],[11,"clone","","",1,null],[11,"clone","","",4,null],[11,"clone","","",3,null],[11,"fmt","","",0,null],[11,"fmt","","",2,null],[11,"fmt","","",1,null],[11,"fmt","","",4,null],[11,"fmt","","",3,null],[11,"hash","","",0,null],[11,"hash","","",2,null],[11,"hash","","",1,null],[11,"hash","","",4,null],[11,"hash","","",3,null],[11,"as_ref","algorithmia","",42,null],[11,"eq","","",42,null],[11,"from_str","","",42,{"inputs":[{"name":"str"}],"output":{"name":"result"}}],[11,"fmt","","",42,null],[11,"to_socket_addrs","","",42,null],[11,"index","","",42,null],[11,"index","","",42,null],[11,"index","","",42,null],[11,"index","","",42,null],[11,"fmt","","",42,null],[11,"clone","","",42,null],[11,"hash","","",42,null],[11,"partial_cmp","","",42,null],[11,"cmp","","",42,null],[11,"into_url","","",42,null],[11,"fmt","algorithmia::client","",37,null],[11,"drop","","",37,null],[11,"from","","",38,{"inputs":[{"name":"r"}],"output":{"name":"body"}}],[11,"read","","",37,null],[11,"read","","",38,null],[11,"as_str","algorithmia::mime","",1,null],[11,"as_str","","",2,null],[11,"as_str","","",3,null],[11,"as_str","","",4,null],[11,"parse","algorithmia","Parse an absolute URL from a string.",42,{"inputs":[{"name":"str"}],"output":{"name":"result"}}],[11,"join","","Parse a string as an URL, with this URL as the base URL.",42,null],[11,"options","","Return a default `ParseOptions` that can fully configure the URL parser.",42,{"inputs":[],"output":{"name":"parseoptions"}}],[11,"as_str","","Return the serialization of this URL.",42,null],[11,"into_string","","Return the serialization of this URL.",42,null],[11,"origin","","Return the origin of this URL (https://url.spec.whatwg.org/#origin)",42,null],[11,"scheme","","Return the scheme of this URL, lower-cased, as an ASCII string without the &#39;:&#39; delimiter.",42,null],[11,"has_authority","","Return whether the URL has an &#39;authority&#39;,\nwhich can contain a username, password, host, and port number.",42,null],[11,"cannot_be_a_base","","Return whether this URL is a cannot-be-a-base URL,\nmeaning that parsing a relative URL string with this URL as the base will return an error.",42,null],[11,"username","","Return the username for this URL (typically the empty string)\nas a percent-encoded ASCII string.",42,null],[11,"password","","Return the password for this URL, if any, as a percent-encoded ASCII string.",42,null],[11,"has_host","","Equivalent to `url.host().is_some()`.",42,null],[11,"host_str","","Return the string representation of the host (domain or IP address) for this URL, if any.",42,null],[11,"host","","Return the parsed representation of the host for this URL.\nNon-ASCII domain labels are punycode-encoded per IDNA.",42,null],[11,"domain","","If this URL has a host and it is a domain name (not an IP address), return it.",42,null],[11,"port","","Return the port number for this URL, if any.",42,null],[11,"port_or_known_default","","Return the port number for this URL, or the default port number if it is known.",42,null],[11,"with_default_port","","If the URL has a host, return something that implements `ToSocketAddrs`.",42,null],[11,"path","","Return the path for this URL, as a percent-encoded ASCII string.\nFor cannot-be-a-base URLs, this is an arbitrary string that doesn’t start with &#39;/&#39;.\nFor other URLs, this starts with a &#39;/&#39; slash\nand continues with slash-separated path segments.",42,null],[11,"path_segments","","Unless this URL is cannot-be-a-base,\nreturn an iterator of &#39;/&#39; slash-separated path segments,\neach as a percent-encoded ASCII string.",42,null],[11,"query","","Return this URL’s query string, if any, as a percent-encoded ASCII string.",42,null],[11,"query_pairs","","Parse the URL’s query string, if any, as `application/x-www-form-urlencoded`\nand return an iterator of (key, value) pairs.",42,null],[11,"fragment","","Return this URL’s fragment identifier, if any.",42,null],[11,"set_fragment","","Change this URL’s fragment identifier.",42,null],[11,"set_query","","Change this URL’s query string.",42,null],[11,"query_pairs_mut","","Manipulate this URL’s query string, viewed as a sequence of name/value pairs\nin `application/x-www-form-urlencoded` syntax.",42,null],[11,"set_path","","Change this URL’s path.",42,null],[11,"path_segments_mut","","Return an object with methods to manipulate this URL’s path segments.",42,null],[11,"set_port","","Change this URL’s port number.",42,null],[11,"set_host","","Change this URL’s host.",42,null],[11,"set_ip_host","","Change this URL’s host to the given IP address.",42,null],[11,"set_password","","Change this URL’s password.",42,null],[11,"set_username","","Change this URL’s username.",42,null],[11,"set_scheme","","Change this URL’s scheme.",42,null],[11,"from_file_path","","Convert a file name as `std::path::Path` into an URL in the `file` scheme.",42,{"inputs":[{"name":"p"}],"output":{"name":"result"}}],[11,"from_directory_path","","Convert a directory name as `std::path::Path` into an URL in the `file` scheme.",42,{"inputs":[{"name":"p"}],"output":{"name":"result"}}],[11,"to_file_path","","Assuming the URL is in the `file` scheme or similar,\nconvert its path to an absolute `std::path::Path`.",42,null],[11,"apply_str","algorithmia::algo","",14,null],[11,"apply_json","","",14,null],[11,"apply_bytes","","",14,null],[11,"apply","","The default implementation of this method calls\n`apply_str`, `apply_json`, or `apply_bytes` based on the input type.",14,null],[11,"new","algorithmia::client","Creates a new response from a server.",37,{"inputs":[{"name":"url"},{"name":"box"}],"output":{"name":"result"}}],[11,"with_message","","Creates a new response received from the server on the given `HttpMessage`.",37,{"inputs":[{"name":"url"},{"name":"box"}],"output":{"name":"result"}}],[11,"status_raw","","Get the raw status code and reason.",37,null]],"paths":[[3,"Mime"],[4,"SubLevel"],[4,"TopLevel"],[4,"Value"],[4,"Attr"],[3,"Algorithm"],[3,"AlgoRef"],[3,"AlgoMetadata"],[3,"AlgoResponse"],[4,"AlgoInput"],[4,"AlgoOutput"],[4,"Version"],[3,"AlgoOptions"],[8,"DecodedEntryPoint"],[8,"EntryPoint"],[3,"XDataType"],[3,"XErrorMessage"],[3,"DeletedResult"],[4,"DataType"],[4,"DataObject"],[3,"DirectoryUpdated"],[3,"DirectoryDeleted"],[3,"DataAcl"],[3,"DirectoryListing"],[3,"DataFileEntry"],[4,"ReadAcl"],[4,"DirEntry"],[3,"DataDir"],[3,"FileAdded"],[3,"FileDeleted"],[3,"DataResponse"],[3,"DataFile"],[8,"HasDataPath"],[3,"DataPath"],[3,"ApiError"],[3,"ApiErrorResponse"],[4,"Error"],[3,"Response"],[4,"Body"],[3,"HttpClient"],[4,"ApiAuth"],[3,"Algorithmia"],[3,"Url"]]};
initSearch(searchIndex);
