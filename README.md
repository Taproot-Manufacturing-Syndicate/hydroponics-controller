This project implements an ebb and flow hydroponic system.
![](/ebb-flow.jpg)
![](/ebb-flow.png)


# System Architecture

A tank holds the nutrient solution.  The tank is opaque to discourage
algae growth, and made of a food safe material.

A submersible pump in the tank lifts the nutrient solution to the
plant tray.

The plant tray sits above the nutrient solution tank and holds the plants.
It has an overflow drain at the top of the plant roots with a higher flow
rate than the pump, so if the pump gets stuck on water just circulates
between the tray and the tank with no spill or overflow.

In the "ebb" part of the cycle the pump is powered off and all the
nutrient solution is at rest in the tank, at the bottom of the system.

In the "flow" part of the cycle the pump is powered on and the nutrient
solution moves up to the plant tray, flooding the roots of the plants.
The level of nutrient solution in the tray rises until it reaches the
overflow drain, at which point any extra drains back down to the tank.
Once the plant tray is filled, the nutrient solution recirculates between
the tank and the tray until the pump turns off.  When the pump turns off
all the nutrient solution drains back down to the tank, either through
the pump or through a dedicated slow drain in the floor of the plant tray.

The system is designed to fail safe, so that if anything goes wrong the
nutrient solution just drains down to the reservoir.

* The plant tray overflow valve has a higher flow rate than the pump.

* There is a large enough volume of nutrient solution in the system that
  with the plant tray full, the pump in the tank is still submerged.
