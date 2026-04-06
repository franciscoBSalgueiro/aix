#include "aixchess_functions.hpp"

#include <cstdint>
#include <cstddef>

namespace duckdb {

struct PieceCountSquares {
	bool valid;

	Bitboards bb;

	static void AssignResult(Vector &result, idx_t i, PieceCountSquares value) {
		if (!value.valid) {
			FlatVector::SetNull(result, i, true);
			return;
		}

		auto &entries = StructVector::GetEntries(result);
		FlatVector::GetData<uint8_t>(*entries[0])[i] = static_cast<uint8_t>(bits::popcount(value.bb.w_q));
		FlatVector::GetData<uint8_t>(*entries[1])[i] = static_cast<uint8_t>(bits::popcount(value.bb.w_r));
		FlatVector::GetData<uint8_t>(*entries[2])[i] = static_cast<uint8_t>(bits::popcount(value.bb.w_b));
		FlatVector::GetData<uint8_t>(*entries[3])[i] = static_cast<uint8_t>(bits::popcount(value.bb.w_n));
		FlatVector::GetData<uint8_t>(*entries[4])[i] = static_cast<uint8_t>(bits::popcount(value.bb.w_p));
		FlatVector::GetData<uint8_t>(*entries[5])[i] = static_cast<uint8_t>(bits::popcount(value.bb.b_q));
		FlatVector::GetData<uint8_t>(*entries[6])[i] = static_cast<uint8_t>(bits::popcount(value.bb.b_r));
		FlatVector::GetData<uint8_t>(*entries[7])[i] = static_cast<uint8_t>(bits::popcount(value.bb.b_b));
		FlatVector::GetData<uint8_t>(*entries[8])[i] = static_cast<uint8_t>(bits::popcount(value.bb.b_n));
		FlatVector::GetData<uint8_t>(*entries[9])[i] = static_cast<uint8_t>(bits::popcount(value.bb.b_p));
	}
};

inline void PieceCountsAtPosition(DataChunk &args, ExpressionState &state, Vector &result) {
	GenericExecutor::ExecuteBinary<PrimitiveType<string_t>, PrimitiveType<int32_t>, PieceCountSquares>(
	    args.data[0], args.data[1], result, args.size(), [&](PrimitiveType<string_t> game, PrimitiveType<int32_t> pos) {
		    PieceCountSquares pcs;
		    diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.val.GetData()), game.val.GetSize()};

		    auto bitboards_result = Game::pieces_at_position(data, pos.val);
		    const auto bitboards_opt =
		        UnwrapOptionalDecoded<Bitboards>(std::move(bitboards_result), "piece_counts_at_position");

		    if (!bitboards_opt.has_value()) {
			    pcs.valid = false;
			    return pcs;
		    }

		    pcs.valid = true;
		    pcs.bb = *bitboards_opt;

		    return pcs;
	    });
}

inline void PieceCountsAtPositionFromFen(DataChunk &args, ExpressionState &state, Vector &result) {
	auto count = args.size();
	result.SetVectorType(VectorType::FLAT_VECTOR);
	auto &validity = FlatVector::Validity(result);

	UnifiedReader<string_t> game_reader;
	UnifiedReader<int32_t> pos_reader;
	UnifiedReader<string_t> fen_reader;
	game_reader.Init(args.data[0], count);
	pos_reader.Init(args.data[1], count);
	fen_reader.Init(args.data[2], count);

	for (idx_t i = 0; i < count; i++) {
		if (game_reader.IsNull(i) || pos_reader.IsNull(i)) {
			validity.SetInvalid(i);
			continue;
		}

		auto game = game_reader.Get(i);
		auto pos = pos_reader.Get(i);
		diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};

		auto bitboards_result = fen_reader.IsNull(i)
		                         ? Game::pieces_at_position(data, pos)
		                         : Game::pieces_at_position_from_fen(
		                               data,
		                               pos,
		                               std::string_view(fen_reader.Get(i).GetData(), fen_reader.Get(i).GetSize()));
		const auto bitboards_opt =
		    UnwrapOptionalDecoded<Bitboards>(std::move(bitboards_result), "piece_counts_at_position");

		if (!bitboards_opt.has_value()) {
			validity.SetInvalid(i);
			continue;
		}

		PieceCountSquares pcs;
		pcs.valid = true;
		pcs.bb = *bitboards_opt;
		PieceCountSquares::AssignResult(result, i, pcs);
	}
}

void Register_PieceCountsAtPosition(ExtensionLoader &loader) {
	child_list_t<LogicalType> piece_counts_children;
	piece_counts_children.push_back(std::make_pair("wQ", LogicalType::UTINYINT));
	piece_counts_children.push_back(std::make_pair("wR", LogicalType::UTINYINT));
	piece_counts_children.push_back(std::make_pair("wB", LogicalType::UTINYINT));
	piece_counts_children.push_back(std::make_pair("wN", LogicalType::UTINYINT));
	piece_counts_children.push_back(std::make_pair("wP", LogicalType::UTINYINT));
	piece_counts_children.push_back(std::make_pair("bQ", LogicalType::UTINYINT));
	piece_counts_children.push_back(std::make_pair("bR", LogicalType::UTINYINT));
	piece_counts_children.push_back(std::make_pair("bB", LogicalType::UTINYINT));
	piece_counts_children.push_back(std::make_pair("bN", LogicalType::UTINYINT));
	piece_counts_children.push_back(std::make_pair("bP", LogicalType::UTINYINT));

	auto piece_counts_pos_function =
	    ScalarFunction("piece_counts_at_position", {LogicalType::BLOB, LogicalType::INTEGER},
	                   LogicalType::STRUCT(piece_counts_children), PieceCountsAtPosition);
	loader.RegisterFunction(piece_counts_pos_function);

	auto piece_counts_pos_from_fen_function =
	    ScalarFunction("piece_counts_at_position", {LogicalType::BLOB, LogicalType::INTEGER, LogicalType::VARCHAR},
	                   LogicalType::STRUCT(piece_counts_children), PieceCountsAtPositionFromFen);
	loader.RegisterFunction(piece_counts_pos_from_fen_function);
}

} // namespace duckdb