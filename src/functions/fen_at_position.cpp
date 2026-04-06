#include "aixchess_functions.hpp"

namespace duckdb {

namespace {

inline void FenAtPosition(DataChunk &args, ExpressionState &state, Vector &result) {
	BinaryExecutor::ExecuteWithNulls<string_t, int32_t, string_t>(
	    args.data[0], args.data[1], result, args.size(),
	    [&](string_t game, int32_t pos, ValidityMask &mask, idx_t idx) {
		    diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
		    auto fen_result = Game::fen_at_position(data, pos);
		    auto fen_opt = UnwrapOptionalDecoded<std::string>(std::move(fen_result), "fen_at_position");

		    if (!fen_opt.has_value()) {
			    mask.SetInvalid(idx);
			    return string_t();
		    }

		    return StringVector::AddString(result, *fen_opt);
	    });
}

inline void FenAtPositionFromFen(DataChunk &args, ExpressionState &state, Vector &result) {
	auto count = args.size();
	result.SetVectorType(VectorType::FLAT_VECTOR);
	auto &validity = FlatVector::Validity(result);

	UnifiedReader<string_t> game_reader;
	UnifiedReader<int32_t> pos_reader;
	UnifiedReader<string_t> fen_reader;
	game_reader.Init(args.data[0], count);
	pos_reader.Init(args.data[1], count);
	fen_reader.Init(args.data[2], count);

	auto out = FlatVector::GetData<string_t>(result);
	for (idx_t i = 0; i < count; i++) {
		if (game_reader.IsNull(i) || pos_reader.IsNull(i)) {
			validity.SetInvalid(i);
			continue;
		}

		auto game = game_reader.Get(i);
		auto pos = pos_reader.Get(i);
		diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};

		auto fen_result = fen_reader.IsNull(i)
		                      ? Game::fen_at_position(data, pos)
		                      : Game::fen_at_position_from_fen(
		                            data,
		                            pos,
		                            std::string_view(fen_reader.Get(i).GetData(), fen_reader.Get(i).GetSize()));
		auto fen_opt = UnwrapOptionalDecoded<std::string>(std::move(fen_result), "fen_at_position");

		if (!fen_opt.has_value()) {
			validity.SetInvalid(i);
			continue;
		}

		out[i] = StringVector::AddString(result, *fen_opt);
	}
}

} // namespace

void Register_FenAtPosition(ExtensionLoader &loader) {
	auto fen_pos_function = ScalarFunction("fen_at_position", {LogicalType::BLOB, LogicalType::INTEGER},
	                                       LogicalType::VARCHAR, FenAtPosition);
	loader.RegisterFunction(fen_pos_function);

	auto fen_pos_from_fen_function =
	    ScalarFunction("fen_at_position", {LogicalType::BLOB, LogicalType::INTEGER, LogicalType::VARCHAR},
	                   LogicalType::VARCHAR, FenAtPositionFromFen);
	loader.RegisterFunction(fen_pos_from_fen_function);
}

} // namespace duckdb