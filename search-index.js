var searchIndex = {};
searchIndex['algodata'] = {"items":[],"paths":[]};
searchIndex['algo'] = {"items":[],"paths":[]};
searchIndex['algorithmia'] = {"items":[[0,"","algorithmia","Algorithmia client library"],[3,"Service","","The top-level struct for instantiating Algorithmia service endpoints"],[12,"api_key","","",0],[3,"ApiClient","","Internal ApiClient to manage connection and requests: wraps `hyper` client"],[3,"ApiErrorResponse","","Struct for decoding Algorithmia API error responses"],[12,"error","","",1],[12,"stacktrace","","",1],[4,"AlgorithmiaError","","Errors that may be returned by this library"],[13,"ApiError","","Errors returned by the Algorithmia API",2],[13,"HttpError","","HTTP errors encountered by the hyper client",2],[13,"DecoderError","","Errors decoding response json",2],[13,"DecoderErrorWithContext","","Errors decoding response json with additional debugging context",2],[13,"EncoderError","","Errors encoding the request",2],[13,"IoError","","General IO errors",2],[0,"algorithm","","Algorithm module for executing Algorithmia algorithms"],[3,"Algorithm","algorithmia::algorithm","Algorithmia algorithm"],[12,"user","","",3],[12,"repo","","",3],[3,"AlgorithmOutput","","Generic struct for decoding an algorithm response JSON"],[12,"duration","","",4],[12,"result","","",4],[3,"AlgorithmService","","Service endpoint for executing algorithms"],[12,"service","","",5],[12,"algorithm","","",5],[6,"AlgorithmResult","","Result type for generic `AlgorithmOutput` when calling `exec`"],[6,"AlgorithmJsonResult","","Result type for the raw JSON returned by calling `exec_raw`"],[11,"fmt","","",4],[11,"decode","","",4],[11,"new","","Instantiate `AlgorithmService` directly - alternative to `Service::algorithm`",5],[11,"exec","","Execute an algorithm with type-safety\ninput_data must be JSON-encodable\n    use `#[derive(RustcEncodable)]` for complex input",5],[11,"exec_raw","","Execute an algorithm with with string input and receive the raw JSON response",5],[0,"collection","algorithmia","Algorithm module for managing Algorithmia Data Collections"],[3,"Collection","algorithmia::collection","Algorithmia data collection"],[12,"user","","",6],[12,"name","","",6],[3,"CollectionAcl","","Permissions for a data collection"],[12,"read_w","","Readable by world",7],[12,"read_g","","Readable by group",7],[12,"read_u","","Readable by user",7],[12,"read_a","","Readable by user's algorithms regardless who runs them",7],[3,"CollectionCreated","","Response when creating a new collection"],[12,"collection_id","","",8],[12,"object_id","","",8],[12,"collection_name","","",8],[12,"username","","",8],[12,"acl","","",8],[3,"CollectionShow","","Response when querying an existing collection"],[12,"username","","",9],[12,"collection_name","","",9],[12,"files","","",9],[3,"CollectionFileAdded","","Response when adding a file to a collection"],[12,"result","","",10],[3,"CollectionService","","Service endpoint for managing Algorithmia data collections"],[12,"service","","",11],[12,"collection","","",11],[6,"CollectionShowResult","",""],[6,"CollectionCreatedResult","",""],[6,"CollectionFileAddedResult","",""],[11,"fmt","","",7],[11,"decode","","",7],[11,"fmt","","",8],[11,"decode","","",8],[11,"fmt","","",9],[11,"decode","","",9],[11,"fmt","","",10],[11,"decode","","",10],[11,"new","","Instantiate `CollectionService` directly - alternative to `Service::collection`",11],[11,"show","","Display collection details if it exists",11],[11,"create","","Create a collection",11],[11,"upload_file","","Upload a file to an existing collection",11],[11,"write_file","","Write a file (raw bytes) directly to a data collection",11],[7,"API_BASE_URL","algorithmia",""],[11,"fmt","","",2],[11,"fmt","","",1],[11,"decode","","",1],[11,"new","","Instantiate a new Service",0],[11,"api_client","","Instantiate a new hyper client - used internally by instantiating new api_client for every request",0],[11,"algorithm","","Instantiate an `AlgorithmService` from this `Service`",0],[11,"collection","","Instantiate a `CollectionService` from this `Service`",0],[11,"decode_to_result","","Helper to standardize decoding to a specific Algorithmia Result type",0],[11,"new","","Instantiate an ApiClient - creates a new `hyper` client",12],[11,"get","","Helper to make Algorithmia GET requests with the API key",12],[11,"post","","Helper to make Algorithmia POST requests with the API key",12],[11,"post_json","","Helper to POST JSON to Algorithmia with the correct Mime types",12],[11,"clone","","",0],[11,"from_error","","",2],[11,"from_error","","",2],[11,"from_error","","",2],[11,"from_error","","",2]],"paths":[[3,"Service"],[3,"ApiErrorResponse"],[4,"AlgorithmiaError"],[3,"Algorithm"],[3,"AlgorithmOutput"],[3,"AlgorithmService"],[3,"Collection"],[3,"CollectionAcl"],[3,"CollectionCreated"],[3,"CollectionShow"],[3,"CollectionFileAdded"],[3,"CollectionService"],[3,"ApiClient"]]};
initSearch(searchIndex);
