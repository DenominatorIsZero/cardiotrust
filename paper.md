---
title: 'CardioTRust: A Research Framework For Cardiac Electrophysiological Simulation and Non-invasive Electroanatomical Mapping'
tags:
  - Rust
  - cardiac electrophysiology
  - magnetocardiography
  - arrhythmia localization
  - state-space modeling
  - medical imaging
  - electroanatomical mapping
  - gradientt-based optimization
authors:
  - name: Erik Engelhardt
    orcid: 0000-0002-7012-7707
    affiliation: 1
  - name: Norbert Frey
    affiliation: 2
  - name: Gerhard Schmidt
    orcid: 0000-0002-6128-4831
    affiliation: 1
affiliations:
  - name: Department of Electrical Information Engineering, Faculty of Engineering, Kiel University
    index: 1
  - name: Department of Internal Medicine III, University Medical Center Heidelberg
    index: 2
date: 17 January 2025
bibliography: paper.bib
---

# Summary

CardioTRust is a standalone graphical application for cardiac electrophysiological simulation and non-invasive electroanatomical mapping, specifically for the localization of arrhythmogenic tissue. The software provides researchers with an interactive environment for estimating myocardial current density distributions from simulated magnetocardiographic measurements. By modeling the propagation of electrical activity through the heart using all-pass filters and voxel-based anatomical structures, CardioTrust enables users to non-invasively study cardiac electrical activity and identify potential arrhythmogenic regions. CardioTRust currently supports a gradient based optimization algorithm for the inverse problem, as well as a reference pseudo-inverse solution. Because the algorithm is still under technical development, the current version of the software exclusively works with simulated data and is not yet suitable for clinical use. However, it provides a promising foundation for future advancements in cardiac electrophysiology research, by providing a robust way to reproduce published results and easy extensibility for testing new algorithms.

# Statement of Need

Electroanatomical mapping (EAM) is essential for diagnosing and treating cardiac arrhythmias, particularly those arising from tissue scarring after myocardial infarction. Traditional EAM involves invasive catheter procedures, which carry inherent risks such as bleeding and infection. While non-invasive alternatives exist, they face challenges due to the limited number of available sensors and the ill-posed nature of the inverse problem. CardioTRust addresses these limitations through a novel state-space approach that combines forward modeling with temporal constraints, enabling more accurate reconstruction of cardiac electrical activity from non-invasive measurements.

Alternative approaches either rely on exhaustive parameter searches with detailed physiological models, which scale poorly, or require extensive training data from simultaneous invasive and non-invasive measurements, which is difficult to obtain for human subjects. CardioTRust provides a middle ground by using a differentiable model that can be optimized using gradient descent while maintaining interpretable parameters with electrophysiological meaning without relying on invasive training data.

Furthermore, while other software solutions like OpenCARP exist, that provide way more sophisticated forward modelling capabilities, they lack the ability to estimate current density distributions from magnetocardiographic measurements. CardioTRust addresses this limitation by providing a comprehensive solution for both forward and inverse modeling, enabling researchers to explore different algorithms and approaches to solve the inverse problem.

# Functionality

CardioTRust implements a novel state-space approach to the inverse problem of cardiac current density estimation. Unlike existing solutions that either use parameter searches with detailed physiological models or require extensive training data, CardioTRust employs a differentiable model with interpretable parameters that can be optimized using gradient descent.

The software's core innovation lies in its use of all-pass filters to model current propagation between voxels, allowing for arbitrary propagation paths and velocities while maintaining differentiability. This enables the use of gradient-based optimization techniques instead of exhaustive parameter searches, making it computationally feasible to handle the large number of parameters needed to model pathological tissue states.

The algorithm consists of nested optimization loops: an outer loop refines the model parameters using gradient descent, while an inner loop employs a Kalman filter for state estimation. This approach leverages both spatial and temporal information from the measurements, improving the accuracy of the reconstruction compared to methods that treat each time step independently.

The software is implemented in Rust with a modern GUI framework, providing researchers with an accessible tool for investigating non-invasive cardiac mapping approaches. All visualization and analysis features are designed to support rapid iteration and evaluation of the estimation results.

# Acknowledgments

This research was funded by the German Research Foundation (Deutsche Forschungsgemeinschaft, DFG) through the Collaborative Research Center CRC 1261 "Magnetoelectric Sensors: From Composite Materials to Biomagnetic Diagnostics".

The development of CardioTrust has benefited from feedback and discussions with medical professionals at the University Medical Center Heidelberg and Kepler University Hospital. We also thank the volunteer participants who helped validate the measurement setup.

The 3D visualization capabilities of CardioTRust build upon the Bevy game engine and the egui immediate mode GUI library, both open source projects with active communities that have been instrumental in creating an interactive research tool.

# References
