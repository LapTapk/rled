#!/usr/bin/env escript
%%! -noshell

main([BeamPath, OutputPath]) ->
    Disasm =
        case beam_disasm:file(BeamPath) of
            {error, _, Error} ->
                io:format("Disassembly failed: ~p~n", [Error]),
                halt(2);
            RawTerm ->
                RawTerm
        end,

    WriteData = term_to_binary(Disasm),

    case file:write_file(OutputPath, WriteData) of
        ok ->
            io:format("ETF disassembly written to ~s~n", [OutputPath]),
            io:format("~p~n", [Disasm]),
            halt(0);
        {error, FileErr} ->
            io:format("Error writing to file: ~p~n", [FileErr]),
            halt(3)
    end;
main(_) ->
    io:format("Usage: beam_disasm <input.beam> <output.txt>~n"),
    halt(1).
