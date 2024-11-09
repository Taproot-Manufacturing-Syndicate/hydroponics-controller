This project implements an ebb and flow hydroponic system.
![](/ebb-flow.jpg)
![](/ebb-flow.png)


# System Architecture

A tank holds the nutrient solution.

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


# System variables

The hydroponic system is characterized by several environmental variables
and controls.

Plant environment:
* light level
* temperature
* air humidity
* carbon dioxide level

Hydroponic system main controls:
* the main nutrient pump
* air circulation
* air and/or nutrient solution heater/cooler

Nutrient solution quality:
* concentration of nutrients
* pH
* dissolved oxygen
* turbidity (to detect algae growth and other dysfunction)


# Bill of Materials


## Nutrient solution tank

Opaque, with lid, to reduce the light that encourages algae growth.

Made of a food safe material: high-density polyethylene (HDPE),
low-density polyethylene (LDPE), polypropylene (PP), or polycarbonate
(PC).  No ABS.  Sometimes the manufacturer will report what material
a tank is made from, sometimes you can look for the recycling sign: "1
PETE" (polyethylene terephthalate), "2 HDPE" (high density polyethylene),
"4 LDPE" (low density polyethylene), and "5 PP" (polypropylene) are all
good options.

A food safe 5 gallon bucket would work well.  You can find
this at home brewing stores, or at Lowe's: $7, plus $3 for the lid:
<https://www.lowes.com/pd/Leaktite-5-gal-70mil-Food-Safe-White-Bucket/5013212247>
HDPE, says the manufacturer:
<https://www.leaktite.com/products/5-gal-foodsafe-70-mil-white>


## Main pump

The main pump sits submerged in the nutrient solution tank.

One option is the Active Aqua 250 GPH, $29.  I like
that they publish GPH vs lift height curves.
<https://hydrobuilder.com/active-aqua-submersible-water-pumps.html>

The pump is powered by AC via a computer controlled power outlet.


## Plant tray

This is an open-topped tray that the plants sit in.

Made of food safe materials (see notes in the Nutrient Solution Tank
section).

Again a food safe 5 gallon bucket can work, with the walls cut to the
proper height.

There are also trays specifically made for hydroponics, but food safe
ones can be expensive.


## Plant tray fixtures

The plant tray has two or three holes in the bottom:

* A slow drain (slower than the pump) at the lowest point on the floor.
  This may not be necessary, if the pump can act as a drain when the
  power is turned off.

* A fast drain (faster than the pump) with a riser on the intake.
  This provides overflow prevention (unless it clogs).

* A faucet or spigot, fed by the main pump.

Options:

* <https://hydrobuilder.com/grow1-ebb-flow-kit.html>

* <https://hydrobuilder.com/active-aqua-fill-drain-combo-kit.html>


## Tubing

We'll use tubing to connect the pump to the fill fixture of the plant
tray, and connect the fast overflow drain fixture back to the nutrient
solution tank.

The dimensions of the tubing have to match the pump and the fixtures
obviously.


## Nutrient solution controller

This is a computer that monitors and controls a couple of variables
relating to the nutrient solution.  It reports periodic telemetry,
and any events that need human attention.

Nutrient solution sensors include:

* pH

    * PH-4502C sensor + probe
      <https://cimpleo.com/blog/arduino-ph-meter-using-ph-4502c/>
      $21 <https://www.ebay.com/itm/354713592722>

    * Atlas Scientific "Surveyor" analog pH kit: $69,
      produces a voltage 0.265-2.745 for pH 14 to
      0. <https://atlas-scientific.com/kits/surveyor-analog-ph-kit>

* nutrient concentration (as estimated by Electrical Conductivity or
  Total Dissolved Solids)

    * DFRobot TDS Sensor: $12
      <https://wiki.dfrobot.com/Gravity__Analog_TDS_Sensor___Meter_For_Arduino_SKU__SEN0244>

The nutrient solution controller uses volumetric pumps driven by stepper
motors to add control fluids to the nutrient solution tank.  The control
fluids are:

* pH up <https://hydrobuilder.com/microbe-life-hydroponics-ph-up.html>

* pH down <https://hydrobuilder.com/microbe-life-hydroponics-ph-down.html>

* nutrient concentrate


## Volumetric pump

Here's a mostly-printed option for the volumetric/peristaltic pump:

* <https://github.com/DerSchultze/Peristaltic-Controller>

* <https://www.thingiverse.com/thing:254956>

It uses a 28BYJ-48 stepper gear motor.

The steppers are cheap and easy to get, e.g.:

* <https://www.ebay.com/itm/224283233677>

* <https://www.amazon.com/10sets-28BYJ-48-ULN2003-Stepper-Driver/dp/B0CZT299DT>

Good description of the motor and driver here:

* <https://lastminuteengineers.com/28byj48-stepper-motor-arduino-tutorial/>

* <https://components101.com/motors/28byj-48-stepper-motor>


## Lights

??

The lights are powered by AC (like the pump).


## Computer controlled power outlets

These are AC power outlets that can be controlled from a computer.

Ideally should report current draw for diagnostics.

A good inexpensive option here is the [Sonoff S31 with Tasmota
firmware](https://tasmota.github.io/docs/devices/Sonoff-S31/).

We need one for the pump and one for the lights.


## Grow controller

This is a computer that sequences the main pump and lights.
