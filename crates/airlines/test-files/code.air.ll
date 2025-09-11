; ModuleID = 'code.air'
source_filename = "code.metal"
target datalayout = "e-p:64:64:64-i1:8:8-i8:8:8-i16:16:16-i32:32:32-i64:64:64-f32:32:32-f64:64:64-v16:16:16-v24:32:32-v32:32:32-v48:64:64-v64:64:64-v96:128:128-v128:128:128-v192:256:256-v256:256:256-v512:512:512-v1024:1024:1024-n8:16:32"
target triple = "air64-apple-macosx15.0.0"

@_ZL5color = internal unnamed_addr addrspace(2) constant [3 x <3 x float>] [<3 x float> <float 1.000000e+00, float 0.000000e+00, float 0.000000e+00>, <3 x float> <float 0.000000e+00, float 1.000000e+00, float 0.000000e+00>, <3 x float> <float 0.000000e+00, float 0.000000e+00, float 1.000000e+00>], align 16
@_ZL8position = internal unnamed_addr addrspace(2) constant [3 x <2 x float>] [<2 x float> <float 0.000000e+00, float -5.000000e-01>, <2 x float> splat (float 5.000000e-01), <2 x float> <float -5.000000e-01, float 5.000000e-01>], align 8

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define <{ <3 x float>, <4 x float> }> @vertexMain(i32 noundef %0) local_unnamed_addr #0 {
  %2 = zext i32 %0 to i64
  %3 = getelementptr inbounds [3 x <3 x float>], ptr addrspace(2) @_ZL5color, i64 0, i64 %2
  %4 = load <3 x float>, ptr addrspace(2) %3, align 16, !tbaa !22
  %5 = getelementptr inbounds [3 x <2 x float>], ptr addrspace(2) @_ZL8position, i64 0, i64 %2
  %6 = load <2 x float>, ptr addrspace(2) %5, align 8, !tbaa !22
  %7 = shufflevector <2 x float> %6, <2 x float> poison, <4 x i32> <i32 0, i32 1, i32 poison, i32 poison>
  %8 = shufflevector <4 x float> %7, <4 x float> <float poison, float poison, float 0.000000e+00, float 1.000000e+00>, <4 x i32> <i32 0, i32 1, i32 6, i32 7>
  %9 = insertvalue <{ <3 x float>, <4 x float> }> undef, <3 x float> %4, 0
  %10 = insertvalue <{ <3 x float>, <4 x float> }> %9, <4 x float> %8, 1
  ret <{ <3 x float>, <4 x float> }> %10
}

attributes #0 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) "approx-func-fp-math"="true" "frame-pointer"="all" "min-legal-vector-width"="0" "no-builtins" "no-infs-fp-math"="true" "no-nans-fp-math"="true" "no-signed-zeros-fp-math"="true" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "unsafe-fp-math"="true" }

!llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8}
!air.vertex = !{!9}
!air.compile_options = !{!15, !16, !17}
!llvm.ident = !{!18}
!air.version = !{!19}
!air.language_version = !{!20}
!air.source_file_name = !{!21}

!0 = !{i32 2, !"SDK Version", [2 x i32] [i32 15, i32 5]}
!1 = !{i32 1, !"wchar_size", i32 4}
!2 = !{i32 7, !"frame-pointer", i32 2}
!3 = !{i32 7, !"air.max_device_buffers", i32 31}
!4 = !{i32 7, !"air.max_constant_buffers", i32 31}
!5 = !{i32 7, !"air.max_threadgroup_buffers", i32 31}
!6 = !{i32 7, !"air.max_textures", i32 128}
!7 = !{i32 7, !"air.max_read_write_textures", i32 8}
!8 = !{i32 7, !"air.max_samplers", i32 16}
!9 = !{ptr @vertexMain, !10, !13}
!10 = !{!11, !12}
!11 = !{!"air.vertex_output", !"user(locn0)", !"air.arg_type_name", !"float3", !"air.arg_name", !"fragColor"}
!12 = !{!"air.position", !"air.arg_type_name", !"float4", !"air.arg_name", !"mtlPosition"}
!13 = !{!14}
!14 = !{i32 0, !"air.vertex_id", !"air.arg_type_name", !"uint", !"air.arg_name", !"vertexID"}
!15 = !{!"air.compile.denorms_disable"}
!16 = !{!"air.compile.fast_math_enable"}
!17 = !{!"air.compile.framebuffer_fetch_enable"}
!18 = !{!"Apple metal version 32023.620 (metalfe-32023.620)"}
!19 = !{i32 2, i32 7, i32 0}
!20 = !{!"Metal", i32 3, i32 2, i32 0}
!21 = !{!"/Users/ignaciolopezcorvalan/Documents/GitHub/rosemetal/crates/airlines/test-files/code.metal"}
!22 = !{!23, !23, i64 0}
!23 = !{!"omnipotent char", !24, i64 0}
!24 = !{!"Simple C++ TBAA"}
