# Technical information

Use rust

# Information Sources

**Boody weight information: (last 7 days)**

```bash
bodylog measurement list --days 7 --json
```

**Sleep information**

```bash
 bodylog sleep list --days 7 --json
 ```

 **Nutrition information**

 ```
 nutlog report nutrition list --days  7 --json
 ```

 # Task

 I need a report with:

 ## An overview of the current values and trends

 ## Details with:

 ### Body information

 Weight as a function of date. With a linear regression and confidence interval. And rate of cahnge per week and day.

 Same for Body fat percentage, muscle percentage, fat mass, muscle mass and resting metabolic rate.


### Sleep data

Trends and history

### Nutriion data

Total calories as a function of date.

Protein per muscle mass.

Fat per muscle mass.

Total fiber.

Total sugars.

## Report type

It has to be an HTML report with navigation. It starts in the overview and then you can navigate to the details. Mobile first. And it can be served statically.
