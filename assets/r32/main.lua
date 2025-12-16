function start()
    rico:log("Welcome to RICO-32!")
    rico:set_frame_rate(60)
end

function update(dt)
    rico:clear("BLACK")
    rico:print_scr(10, 10, "WHITE", "Hello, World!")
    
    local mouse = rico:mouse()
    if mouse.pressed then
        rico:circle(mouse.x, mouse.y, 5, "RED")
    end
end