(print_int (
    for el (+ el 1) (< el 1000) (case (| (! (% el 3)) (! (% el 5)))
        el
        0
    )
))