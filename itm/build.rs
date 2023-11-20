fn main() {
    let cxx_sources = [
        "../extern/itm/src/ComputeDeltaH.cpp",
        "../extern/itm/src/DiffractionLoss.cpp",
        "../extern/itm/src/FindHorizons.cpp",
        "../extern/itm/src/FreeSpaceLoss.cpp",
        "../extern/itm/src/FresnelIntegral.cpp",
        "../extern/itm/src/H0Function.cpp",
        "../extern/itm/src/InitializeArea.cpp",
        "../extern/itm/src/InitializePointToPoint.cpp",
        "../extern/itm/src/InverseComplementaryCumulativeDistributionFunction.cpp",
        "../extern/itm/src/KnifeEdgeDiffraction.cpp",
        "../extern/itm/src/LineOfSightLoss.cpp",
        "../extern/itm/src/LinearLeastSquaresFit.cpp",
        "../extern/itm/src/LongleyRice.cpp",
        "../extern/itm/src/QuickPfl.cpp",
        "../extern/itm/src/SigmaHFunction.cpp",
        "../extern/itm/src/SmoothEarthDiffraction.cpp",
        "../extern/itm/src/TerrainRoughness.cpp",
        "../extern/itm/src/TroposcatterLoss.cpp",
        "../extern/itm/src/ValidateInputs.cpp",
        "../extern/itm/src/Variability.cpp",
        "../extern/itm/src/itm_area.cpp",
        "../extern/itm/src/itm_p2p.cpp",
        "wrapper/itm-wrapper.cpp",
    ];

    let mut bridge = cxx_build::bridge("src/lib.rs");
    bridge.flag("-std=c++11");
    bridge.include("../extern/itm/include");
    #[cfg(feature = "address_sanitizer")]
    {
        bridge.flag("-fno-omit-frame-pointer");
        bridge.flag("-fsanitize=address");
        bridge.flag("-ggdb");
    }
    for path in &cxx_sources {
        bridge.file(path);
    }
    bridge.compile("itm_wrapper");

    println!("cargo:rerun-if-changed=wrapper/itm-wrapper.cpp");
    println!("cargo:rerun-if-changed=wrapper/itm-wrapper.h");
}
