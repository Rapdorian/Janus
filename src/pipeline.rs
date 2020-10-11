pub mod deferred;
pub mod gbuffer;
pub mod lighting;

// ok here is my thoughts on the pipeline
//
// I want to have a deffered render system it may be easiest to split this into three pipelines one
// of which contains the other two
// GbufferPipeline
// ShaderPipeline
// DefferedPipeline: GbufferPipeline + ShaderPipeline
//
// example run
// fn render(pipe: DeferredPipeline) {
//     pipe.render(mesha); // renders gbuffer for mesha
//     pipe.render(meshb); // renders gbuffer for meshb
//     pipe.flush() // runs shaderpipeline
// }
