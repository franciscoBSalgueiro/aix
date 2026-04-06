#include "aixchess_functions.hpp"

namespace duckdb {

namespace {

inline void ToPgn(DataChunk &args, ExpressionState &state, Vector &result) {
	UnaryExecutor::Execute<string_t, string_t>(args.data[0], result, args.size(), [&](string_t game) {
		diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
		auto pgn_result = Game::to_pgn_string(data);
		auto pgn = UnwrapDecoded<std::string>(std::move(pgn_result), "to_pgn");
		return StringVector::AddString(result, pgn);
	});
}

inline void ToPgnFromFen(DataChunk &args, ExpressionState &state, Vector &result) {
	auto count = args.size();
	result.SetVectorType(VectorType::FLAT_VECTOR);
	auto &validity = FlatVector::Validity(result);

	UnifiedReader<string_t> game_reader;
	UnifiedReader<string_t> fen_reader;
	game_reader.Init(args.data[0], count);
	fen_reader.Init(args.data[1], count);

	auto out = FlatVector::GetData<string_t>(result);
	for (idx_t i = 0; i < count; i++) {
		if (game_reader.IsNull(i)) {
			validity.SetInvalid(i);
			continue;
		}

		auto game = game_reader.Get(i);
		diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};

		auto pgn_result = fen_reader.IsNull(i)
		                      ? Game::to_pgn_string(data)
		                      : Game::to_pgn_string_from_fen(
		                            data,
		                            std::string_view(fen_reader.Get(i).GetData(), fen_reader.Get(i).GetSize()));
		auto pgn = UnwrapDecoded<std::string>(std::move(pgn_result), "to_pgn");
		out[i] = StringVector::AddString(result, pgn);
	}
}

} // namespace

void Register_ToPgn(ExtensionLoader &loader) {
	auto to_pgn_function = ScalarFunction("to_pgn", {LogicalType::BLOB}, LogicalType::VARCHAR, ToPgn);
	loader.RegisterFunction(to_pgn_function);

	auto to_pgn_from_fen_function =
	    ScalarFunction("to_pgn", {LogicalType::BLOB, LogicalType::VARCHAR}, LogicalType::VARCHAR, ToPgnFromFen);
	loader.RegisterFunction(to_pgn_from_fen_function);
}

} // namespace duckdb