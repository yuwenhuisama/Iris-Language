#ifndef _H_IRISABSTRACTSTATEMENTVALIDATEVISITOR_
#define _H_IRISABSTRACTSTATEMENTVALIDATEVISITOR_

class IrisStatement;
class IrisBlock;
class IrisAbstractStatementValidateVisitor {
public:
	virtual bool Visit(IrisStatement* pStatement) = 0;
	virtual bool Visit(IrisBlock* pBlock) = 0;
};

#endif
