
# AureaOS — Planned System Tools

## 1. Purpose and Status

This document defines planned graphical system tools for AureaOS Roma 1.0 and the intended user experience they should provide.

The goal is to make common system administration tasks understandable and accessible without requiring users to install separate vendor applications or use the terminal for ordinary operations.

The tools described here are planned features, not implemented software. Their final capabilities depend on available Linux infrastructure, hardware support, driver capabilities, security requirements, and implementation testing.

## 2. Printer Management

### 2.1 Purpose

Printer management should be integrated into the AureaOS graphical system settings.

Users should be able to install, configure, inspect, and manage compatible printers without needing to understand printing services or install a separate manufacturer application for ordinary tasks.

### 2.2 Supported Connections

The system should support compatible printers connected through:

- USB.
- Wi-Fi.
- Ethernet.
- Other connection methods supported by the available Linux printing stack and device drivers.

AureaOS should reuse CUPS and suitable existing drivers and printing technologies rather than implement a separate printing system.

### 2.3 Adding a Printer

The system settings should provide an **Add Printer** action.

The graphical setup flow should:

- Discover compatible printers where possible.
- Show detected devices and their connection type.
- Allow selection and configuration of a printer.
- Explain when additional driver support is required.
- Report connection, configuration, and installation errors in understandable language.
- Allow the user to select a default printer.

Manual configuration may be offered when automatic discovery is unavailable or insufficient.

### 2.4 Printer Status and Queue

The printer interface should display available information about:

- Connection and readiness status.
- Current printing activity.
- Pending and completed jobs where supported.
- Paused, failed, or cancelled jobs.
- Paper or consumable warnings when reported by the device.
- The default printer.

Users should be able to pause, resume, cancel, and inspect print jobs where supported.

### 2.5 Print Preferences

The interface should expose relevant options supported by the printer and its driver, including:

- Paper size and media type.
- Portrait or landscape orientation.
- Color or grayscale.
- Duplex printing.
- Print quality and resolution.
- Copies and page ranges.
- Supported input trays and finishing options.

Options should reflect the capabilities actually reported by the printing stack.

AureaOS must not display unsupported settings as though they were available.

### 2.6 Ink and Toner Levels

Ink or toner levels should be displayed when the device, driver, or supported management protocol provides reliable information.

If the information is unavailable, the interface should clearly indicate that the level cannot be reported.

The system must not invent consumable levels or assume that every printer supports monitoring.

### 2.7 Manufacturer Software

The default experience should cover common printing and configuration tasks through AureaOS system settings.

A separate manufacturer application should not be required for ordinary supported operations.

Some proprietary devices may require manufacturer-specific drivers, firmware, or utilities for advanced or unsupported features. Such limitations should be explained clearly rather than hidden.

## 3. Scanner Management

### 3.1 Purpose

Scanner management should be integrated into the graphical system experience, with a straightforward workflow for scanning documents and images.

AureaOS should reuse SANE and compatible existing scanning backends and drivers where appropriate.

### 3.2 Device Discovery

The interface should support compatible scanners connected through USB or supported network connections.

Users should be able to:

- Discover and select available scanners.
- Identify the selected device and its connection status.
- Choose available scanner sources.
- Receive understandable error messages.
- Configure compatible devices without using the terminal.

Multifunction printers should be supported as scanners when their hardware and drivers expose scanning capabilities.

### 3.3 Scanning Workflow

The scanning application should provide:

- A preview before final capture.
- Selection of the scan area.
- Page size and orientation controls where supported.
- Resolution settings expressed in DPI.
- Color, grayscale, and black-and-white modes where supported.
- Brightness and contrast adjustments where available.
- Document and image output options.
- Save location and file naming.
- Progress and error feedback.

### 3.4 Document Handling

The initial scope should include saving scans as common image formats and PDF, subject to the selected backend and available conversion support.

Multipage scanning should be supported where practical.

Automatic document feeders and duplex scanning should be available when the scanner, driver, and scanning backend support them.

The system must clearly distinguish supported capabilities from unavailable ones.

### 3.5 Manufacturer Software

Ordinary scanning tasks should be available through the AureaOS graphical experience without requiring a separate vendor application when compatible Linux support exists.

Advanced features may require additional drivers or vendor-specific software. The interface should communicate such limitations clearly.

## 4. Resource Monitor

### 4.1 Purpose

The planned Resource Monitor is a graphical system application for understanding current system activity and identifying applications or processes using significant resources.

It should take inspiration from the clear graphs and process views of Ubuntu's system monitoring tools while following the AureaOS visual design system.

The application should be approachable for everyday users but provide useful detail for experienced users.

### 4.2 Overview

The main overview should present readable summaries and time-series graphs for available resources, including:

- CPU utilization.
- Memory and swap usage.
- Disk activity.
- Network throughput.
- GPU utilization and memory where reliable telemetry is available.

The interface should prioritize clear labels, useful units, readable scales, and a sensible time window.

It should distinguish current usage from total capacity where relevant.

### 4.3 CPU

The CPU view should display overall utilization and per-core information when available.

The application should identify resource-intensive processes and present processor activity over time.

### 4.4 Memory

The memory view should report RAM usage, available memory, and swap usage where applicable.

Labels should help users understand the difference between used, available, cached, and reserved memory when the operating system exposes those distinctions.

### 4.5 Disks

The disk view should present available information about read and write activity, throughput, and relevant device activity.

Disk capacity and free space may be shown where useful, but storage capacity and real-time disk activity must remain clearly distinguished.

### 4.6 Network

The network view should display received and transmitted data rates and their recent history.

The interface should distinguish instantaneous throughput from cumulative transfer totals when both are available.

### 4.7 GPU and Sensors

GPU utilization, GPU memory, temperature, and fan information may be shown when supported by the hardware, driver, and available system interfaces.

The application must not assume that every device exposes these metrics.

Unavailable information should be omitted or clearly marked as unsupported.

### 4.8 Processes

The process view should provide:

- A searchable process list.
- Sorting by CPU, memory, and other available metrics.
- Process names and relevant resource usage.
- Clear distinction between applications and system processes where possible.
- An action to end a process when permitted.

Potentially disruptive actions should require confirmation when appropriate.

System processes must not be terminated automatically merely because they consume substantial resources.

Permission errors and protected processes should be handled safely and explained clearly.

### 4.9 Graphs and Performance

Graphs should update smoothly without unnecessarily consuming the resources they measure.

The application should avoid excessive sampling frequency, expensive visual effects, and unnecessary background work.

The update interval and displayed history should balance responsiveness, clarity, and resource consumption.

### 4.10 Visual Design

The Resource Monitor must use AureaOS Human Interfaces components and follow the shared light and dark themes.

Its design should prioritize:

- Readable graphs and labels.
- Clear resource summaries.
- Consistent spacing and typography.
- Restrained color coding.
- Subtle transitions.
- A clean layout without unnecessary decoration.

It should feel like a native AureaOS application rather than a separate technical utility.

## 5. Integration with AureaOS

Printer and scanner management should be accessible from graphical system settings.

The Resource Monitor should be discoverable through the application launcher and Quaero search.

Where appropriate, the tools should integrate with shared notifications, system status, application navigation, and accessibility services.

They should use established Linux services and APIs instead of duplicating mature system functionality.

## 6. Implementation Priorities

Implementation should proceed in stages:

1. Define the graphical workflows and shared interface components.
2. Integrate existing Linux printing, scanning, and system-monitoring infrastructure.
3. Implement device discovery and capability reporting.
4. Add clear configuration, status, and error handling.
5. Validate functionality against representative hardware and different driver capabilities.
6. Verify keyboard accessibility, visual consistency, and behavior in both themes.

Hardware-dependent features must be tested rather than assumed to work universally.

## 7. Implementation Status

The features described in this document are planned requirements.

Their inclusion does not mean that printer management, scanner management, or the Resource Monitor has already been implemented.

The final scope for the first release will depend on development progress, compatibility testing, and the available system infrastructure.
