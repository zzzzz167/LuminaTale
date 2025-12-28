local reg = lumina.register_layout

reg("left",       {x=0.2, y=1.0})
reg("left_spot",  {x=0.25, y=1.0})
reg("center",     {x=0.5, y=1.0})
reg("right",      {x=0.8, y=1.0})
reg("right_spot", {x=0.75, y=1.0})
reg("mid_air",    {x=0.5, y=0.5, anchor_y=0.5})

lumina.log.info("Layouts loaded.")