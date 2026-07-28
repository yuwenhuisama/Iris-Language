#ifndef _H_IRISBLOCK_
#define _H_IRISBLOCK_

#include "IrisComponents/IrisStatements/IrisStatement.h"
#include "IrisValidator/IIrisStatementValidator.h"
#include "IrisList.h"

class IrisBlock : public IIrisStatementValidator
{
private:
	IrisList<IrisStatement*>* m_pStatements = nullptr;

public:
	IrisBlock(IrisList<IrisStatement*>* pStatements);

	bool Generate();

	~IrisBlock();

	virtual bool Accept(IrisAbstractStatementValidateVisitor* pVisitor) override;
	virtual bool Validate() override;

	friend class IrisOrderStatement;
};

#endif