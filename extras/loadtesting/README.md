# Load Testing

This script is used by [locustio/locust](https://github.com/locustio/locust) to execute a load test.

The test will create/delete cards and vote on them randomly.

## Setup

Before you can run a load test there is some initial setup you need to do.
First we need a board with some ranks and we need both cards_open and voting_open to be true.
Edit locustfile.py and insert the appropriate IDs into the placeholders at the top of the file.

## Run

Run a load test by executing the following command:

```sh
$ locust -H http://127.0.0.1:8000 -f locustfile.py --no-web -c10  -t 60
```

## Example Output


```
[2020-02-08 21:53:40,814] navi/INFO/locust.runners: Hatching and swarming 10 users at the rate 1 users/s (0 users already running)...
[2020-02-08 21:53:41,813] navi/INFO/locust.main: Run time limit set to 60 seconds
[2020-02-08 21:53:41,814] navi/INFO/locust.main: Starting Locust 0.14.4
(...)
Percentage of the requests completed within given times
 Type                 Name                                                           # reqs    50%    66%    75%    80%    90%    95%    98%    99%  99.9% 99.99%   100%
------------------------------------------------------------------------------------------------------------------------------------------------------
 POST                 Create a Card                                                    2017     24     29     32     35     49     77     95    110    140    140    140
 DELETE               Delete a Card                                                    3088     29     34     38     41     58     82    100    130    150    150    150
 GET                  Load Board                                                       3965     30     35     39     42     59     86    110    130    160    190    190
 GET                  Load Cards                                                       3995     17     20     23     26     35     53     69     77     96    100    100
 GET                  Load Ranks                                                       1061     16     20     22     25     38     54     67     79     97    100    100
 POST                 Vote on a Card                                                   3991     32     38     42     45     61     86    110    120    150    160    160
------------------------------------------------------------------------------------------------------------------------------------------------------
 None                 Aggregated                                                      18117     26     32     36     39     53     74    100    120    150    170    190
```