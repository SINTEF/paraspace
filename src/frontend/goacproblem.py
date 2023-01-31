from timelinedsl import *

time_windows = [(280,4000), (8000,15000), (20000,25000), (30000,35000), (40000,45000)]
locations = [0, 1,2,3,4,5,6,7,8,9]
speed = 1
download_time = 6*40
take_pic_time = 50
edges = [
        (0, 1, 100),
        (0, 2, 200),
        (0, 4, 100),
        (0, 5, 200),
        (0, 9, 300),
        (1, 2, 100),
        (1, 6, 200),
        (2, 3, 100),
        (2, 7, 200),
        (2, 8, 300),
        (3, 4, 200),
        (3, 7, 100),
        (3, 8, 200),
        (4, 5, 100),
        (4, 9, 200),
        (5, 6, 100),
        (5, 7, 200),
        (5, 9, 100),
        (6, 7, 100),
        (7, 8, 100),
        (8, 9, 200),
]

edges = edges + [(b,a,l) for a,b,l in edges]


for n_pics in [1,2,3,4,5,6,7,8,9]:
    for n_windows in [1,2,3,4,5]:

        p = Problem()
        p.resource("Antenna", name="Antenna", capacity=1)
        for (start,end) in time_windows[:n_windows]:
            p.fact("Visibility", "Available", start=start, end=end)
        
        loc_timeline = p.timeline("Location", name="loc")
        p.fact("loc", "At(0)",start=0) # Start at location 0

        for loc in locations:
            loc_timeline.state(f"At({loc})")
        for a,b,l in edges:
            loc_timeline.state(f"Going({locations[a]},{locations[b]})", dur=(l,l), conditions=[
                TransitionFrom(f"At({locations[a]})"), 
                TransitionTo(f"At({locations[b]})")])
        
        for loc_idx in range(n_pics):

            # Take the picture
            pic_timeline = p.timeline("HavePicture", name=f"HavePicture{loc_idx}")
            pic_timeline.state("Taking", dur=(take_pic_time, take_pic_time),conditions=[During("loc", f"At({loc_idx})")])
            pic_timeline.state("Done", conditions=[TransitionFrom("Taking")])

            # Download the picture
            dl_timeline = p.timeline("Download", name=f"Download{loc_idx}")
            dl_timeline.state("Downloading", dur=(download_time, download_time), conditions=[
                During(f"HavePicture{loc_idx}", "Done"), 
                During(f"Visibility", "Available"),
                During(f"Antenna", "Available", amount=1)])
            dl_timeline.state("Done", conditions=[TransitionFrom("Downloading")])

            p.goal(f"Download{loc_idx}", "Done")

        p.save_json(f"examples/goac_{n_pics}pics_{n_windows}wind.json")
